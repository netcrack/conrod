//! Everything related to storing the state of user input. This includes the state of any
//! buttons on either the keyboard or the mouse, as well as the position of the mouse.
//! It also includes which widgets, if any, are capturing the keyboard and mouse.
//! This module exists mostly to support the `events::InputProvider` trait.

use input::MouseButton;
use input::keyboard::{NO_MODIFIER, ModifierKey, Key};
use position::Point;
use widget::Index;
use events::UiEvent;

/// The max total number of buttons on a mouse.
pub const NUM_MOUSE_BUTTONS: usize = 9;

/// Describes the position of the mouse when the button was pressed. Will be
/// `None` if the mouse button is currently in the up position.
pub type ButtonDownPosition = Option<Point>;

/// Holds the current state of user input. This includes the state of all buttons on
/// the keyboard and mouse, as well as the position of the mouse. It also includes which
/// widgets, if any, are capturing keyboard and mouse input.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InputState {
    /// A map that stores the up/down state of each button. If the button is down, then
    /// it stores the position of the mouse when the button was first pressed.
    pub mouse_buttons: ButtonMap,
    /// The current position of the mouse.
    pub mouse_position: Point,
    /// Which widget, if any, is currently capturing the keyboard
    pub widget_capturing_keyboard: Option<Index>,
    /// Which widget, if any, is currently capturing the mouse
    pub widget_capturing_mouse: Option<Index>,
    /// Which modifier keys are being held down.
    pub modifiers: ModifierKey,
}

impl InputState {
    /// Returns a fresh new input state
    pub fn new() -> InputState {
        InputState{
            mouse_buttons: ButtonMap::new(),
            mouse_position: [0.0, 0.0],
            widget_capturing_keyboard: None,
            widget_capturing_mouse: None,
            modifiers: NO_MODIFIER,
        }
    }

    /// Updates the input state based on an event.
    pub fn update(&mut self, event: &UiEvent) {
        use input::{Button, Motion, Input};

        match *event {
            UiEvent::Raw(Input::Press(Button::Mouse(mouse_button))) => {
                self.mouse_buttons.set(mouse_button, Some(self.mouse_position));
            },
            UiEvent::Raw(Input::Release(Button::Mouse(mouse_button))) => {
                self.mouse_buttons.set(mouse_button, None);
            },
            UiEvent::Raw(Input::Move(Motion::MouseRelative(x, y))) => {
                self.mouse_position = [x, y];
            },
            UiEvent::Raw(Input::Press(Button::Keyboard(key))) => {
                get_modifier(key).map(|modifier| self.modifiers.insert(modifier));
            },
            UiEvent::Raw(Input::Release(Button::Keyboard(key))) => {
                get_modifier(key).map(|modifier| self.modifiers.remove(modifier));
            },
            UiEvent::WidgetCapturesKeyboard(idx) => {
                self.widget_capturing_keyboard = Some(idx);
            },
            UiEvent::WidgetUncapturesKeyboard(_) => {
                self.widget_capturing_keyboard = None;
            },
            UiEvent::WidgetCapturesMouse(idx) => {
                self.widget_capturing_mouse = Some(idx);
            },
            UiEvent::WidgetUncapturesMouse(_) =>  {
                self.widget_capturing_mouse = None;
            },
            _ => {}
        }
    }

    /// Returns a copy of the InputState relative to the given `position::Point`
    pub fn relative_to(&self, xy: Point) -> InputState {
        InputState {
            mouse_position: ::vecmath::vec2_sub(self.mouse_position, xy),
            ..*self
        }
    }
}

fn get_modifier(key: Key) -> Option<ModifierKey> {
    use input::keyboard::{CTRL, SHIFT, ALT, GUI};

    match key {
        Key::LCtrl | Key::RCtrl => Some(CTRL),
        Key::LShift | Key::RShift => Some(SHIFT),
        Key::LAlt | Key::RAlt => Some(ALT),
        Key::LGui | Key::RGui => Some(GUI),
        _ => None
    }
}

/// Stores the state of all mouse buttons. If the mouse button is down,
/// it stores the position of the mouse when the button was pressed
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ButtonMap {
    button_states: [ButtonDownPosition; NUM_MOUSE_BUTTONS]
}

impl ButtonMap {
    /// Returns a new button map with all states set to `None`
    pub fn new() -> ButtonMap {
        ButtonMap{
            button_states: [None; NUM_MOUSE_BUTTONS]
        }
    }

    /// Sets the state of a specific `MouseButton`
    pub fn set(&mut self, button: MouseButton, point: ButtonDownPosition) {
        let idx = ButtonMap::button_idx(button);
        self.button_states[idx] = point;
    }

    /// Returns the state of a mouse button
    pub fn get(&self, button: MouseButton) -> ButtonDownPosition {
        self.button_states[ButtonMap::button_idx(button)]
    }

    /// Returns the current state of a mouse button, leaving `None` in its place
    pub fn take(&mut self, button: MouseButton) -> ButtonDownPosition {
        self.button_states[ButtonMap::button_idx(button)].take()
    }

    /// If any mouse buttons are currently pressed, will return a tuple containing
    /// both the `MouseButton` that is pressed and the `Point` describing the location of the
    /// mouse when it was pressed.
    pub fn pressed_button(&self) -> Option<(MouseButton, Point)> {
        self.button_states.iter().enumerate().filter(|idx_and_state| idx_and_state.1.is_some())
                .map(|idx_and_state|
                    (ButtonMap::idx_to_button(idx_and_state.0), idx_and_state.1.unwrap()))
                .next()
    }

    fn idx_to_button(idx: usize) -> MouseButton {
        MouseButton::from(idx as u32)
    }
    fn button_idx(button: MouseButton) -> usize {
        u32::from(button) as usize
    }

}



#[test]
fn pressed_button_returns_none_if_no_buttons_are_pressed() {
    let map = ButtonMap::new();
    let pressed = map.pressed_button();
    assert!(pressed.is_none());
}

#[test]
fn pressed_button_should_return_first_pressed_button() {
    let mut map = ButtonMap::new();

    map.set(MouseButton::Right, Some([3.0, 3.0]));
    map.set(MouseButton::X1, Some([5.4, 4.5]));

    let pressed = map.pressed_button();
    assert_eq!(Some((MouseButton::Right, [3.0, 3.0])), pressed);
}

#[test]
fn button_down_should_store_the_point() {
    let mut map = ButtonMap::new();
    let point: ButtonDownPosition = Some([2.0, 5.0]);
    map.set(MouseButton::Left, point);

    assert_eq!(point, map.get(MouseButton::Left));
}

#[test]
fn take_resets_and_returns_current_state() {
    let mut map = ButtonMap::new();
    let point: ButtonDownPosition = Some([2.0, 5.0]);
    map.set(MouseButton::Left, point);

    let taken = map.take(MouseButton::Left);
    assert_eq!(point, taken);
    assert!(map.get(MouseButton::Left).is_none());
}

#[test]
fn input_state_should_be_made_relative_to_a_given_point() {
    let mut state = InputState::new();
    state.mouse_position = [50.0, -10.0];

    let relative_state = state.relative_to([20.0, 20.0]);
    assert_eq!([30.0, -30.0], relative_state.mouse_position);
}
