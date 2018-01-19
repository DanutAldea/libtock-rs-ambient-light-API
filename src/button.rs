use result;
use result::TockResult;
use result::TockValue;
use syscalls;
use util::PhantomLifetime;

const DRIVER_NUMBER: u32 = 0x00003;

mod command_nr {
    pub const COUNT: u32 = 0;
    pub const ENABLE_INTERRUPT: u32 = 1;
    pub const DISABLE_INTERRUPT: u32 = 2;
    pub const READ: u32 = 3;
}

mod subscribe_nr {
    pub const SUBSCRIBE_CALLBACK: u32 = 0;
}

pub struct Buttons<CB> {
    count: usize,
    callback: CB,
}

#[derive(Copy, Clone, Debug)]
pub enum ButtonsError {
    NotSupported,
    SubscriptionFailed,
}

impl Buttons<()> {
    pub fn without_callback() -> TockResult<Self, ButtonsError> {
        Buttons::with_callback(())
    }
}

impl<CB: ButtonCallback> Buttons<CB> {
    pub fn with_callback(callback: CB) -> TockResult<Self, ButtonsError> {
        unsafe extern "C" fn button_callback<CB: ButtonCallback>(
            button_num: usize,
            state: usize,
            _: usize,
            userdata: usize,
        ) {
            let callback = &mut *(userdata as *mut CB);
            callback.callback(button_num, state.into());
        }

        let count = unsafe { syscalls::command(DRIVER_NUMBER, command_nr::COUNT, 0, 0) };

        if count <= 1 {
            return Err(TockValue::Expected(ButtonsError::NotSupported));
        }

        let mut buttons = Buttons {
            count: count as usize,
            callback,
        };

        let return_code = unsafe {
            syscalls::subscribe(
                DRIVER_NUMBER,
                subscribe_nr::SUBSCRIBE_CALLBACK,
                button_callback::<CB>,
                &mut buttons.callback as *mut _ as usize,
            )
        };

        match return_code {
            result::SUCCESS => Ok(buttons),
            result::ENOMEM => Err(TockValue::Expected(ButtonsError::SubscriptionFailed)),
            unexpected => Err(TockValue::Unexpected(unexpected)),
        }
    }
}

pub trait ButtonCallback {
    fn callback(&mut self, button_num: usize, state: ButtonState);
}

impl ButtonCallback for () {
    fn callback(&mut self, _: usize, _: ButtonState) {}
}

impl<F: FnMut(usize, ButtonState)> ButtonCallback for F {
    fn callback(&mut self, button_num: usize, state: ButtonState) {
        self(button_num, state);
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ButtonState {
    Pressed,
    Released,
}

impl From<usize> for ButtonState {
    fn from(state: usize) -> ButtonState {
        match state {
            0 => ButtonState::Released,
            1 => ButtonState::Pressed,
            _ => unreachable!(),
        }
    }
}

impl<CB> Drop for Buttons<CB> {
    fn drop(&mut self) {
        extern "C" fn noop_callback(_: usize, _: usize, _: usize, _: usize) {}

        unsafe {
            syscalls::subscribe(
                DRIVER_NUMBER,
                subscribe_nr::SUBSCRIBE_CALLBACK,
                noop_callback,
                0,
            );
        }

        for mut button in self {
            let _ignore_result = button.disable();
        }
    }
}

impl<'a, CB> IntoIterator for &'a mut Buttons<CB> {
    type Item = ButtonHandle<'a>;
    type IntoIter = ButtonIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ButtonIter {
            curr_button: 0,
            button_count: self.count,
            lifetime: Default::default(),
        }
    }
}

pub struct ButtonIter<'a> {
    curr_button: usize,
    button_count: usize,
    lifetime: PhantomLifetime<'a>,
}

impl<'a> Iterator for ButtonIter<'a> {
    type Item = ButtonHandle<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_button < self.button_count {
            let item = ButtonHandle {
                button_num: self.curr_button,
                lifetime: Default::default(),
            };
            self.curr_button += 1;
            Some(item)
        } else {
            None
        }
    }
}

pub struct ButtonHandle<'a> {
    button_num: usize,
    lifetime: PhantomLifetime<'a>,
}

impl<'a> ButtonHandle<'a> {
    pub fn enable(&mut self) -> TockResult<Button, ButtonError> {
        let return_code = unsafe {
            syscalls::command(
                DRIVER_NUMBER,
                command_nr::ENABLE_INTERRUPT,
                self.button_num as isize,
                0,
            )
        };

        match return_code {
            result::SUCCESS => Ok(Button { handle: self }),
            result::ENOMEM => Err(TockValue::Expected(ButtonError::ActivationFailed)),
            unexpected => Err(TockValue::Unexpected(unexpected)),
        }
    }

    pub fn disable(&mut self) -> TockResult<(), ButtonError> {
        let return_code = unsafe {
            syscalls::command(
                DRIVER_NUMBER,
                command_nr::DISABLE_INTERRUPT,
                self.button_num as isize,
                0,
            )
        };

        match return_code {
            result::SUCCESS => Ok(()),
            result::ENOMEM => Err(TockValue::Expected(ButtonError::ActivationFailed)),
            unexpected => Err(TockValue::Unexpected(unexpected)),
        }
    }
}

pub struct Button<'a> {
    handle: &'a ButtonHandle<'a>,
}

#[derive(Copy, Clone, Debug)]
pub enum ButtonError {
    ActivationFailed,
}

impl<'a> Button<'a> {
    pub fn read(&self) -> ButtonState {
        unsafe {
            ButtonState::from(syscalls::command(
                DRIVER_NUMBER,
                command_nr::READ,
                self.handle.button_num as isize,
                0,
            ) as usize)
        }
    }
}
