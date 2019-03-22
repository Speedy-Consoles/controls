mod triggers;

use std::collections::VecDeque;
use std::collections::vec_deque::Drain;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeMap;
use std::string::ToString;
use std::hash::Hash;
use std::str::FromStr;

use winit::ElementState;
use winit::ButtonId;
use winit::MouseScrollDelta;
use winit::DeviceId;
use winit::DeviceEvent;
use winit::KeyboardInput;

pub use self::triggers::FireTrigger;
pub use self::triggers::HoldableTrigger;
pub use self::triggers::ValueTrigger;
pub use winit::VirtualKeyCode;

#[derive(Debug, PartialEq)]
pub enum MouseWheelDirection {
    Up,
    Down,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SwitchState {
    Active,
    Inactive,
}

pub trait ValueTargetTrait {
    fn base_factor(&self) -> f64;
}

#[derive(Debug)]
pub enum Target<FireTarget, SwitchTarget, ValueTarget>
where FireTarget: FromStr,
      SwitchTarget: FromStr,
      ValueTarget: FromStr,
{
    Fire(FireTarget),
    Switch(SwitchTarget),
    Value(ValueTarget),
}

impl<FireTarget, SwitchTarget, ValueTarget> FromStr
for Target<FireTarget, SwitchTarget, ValueTarget>
where FireTarget: FromStr,
      SwitchTarget: FromStr,
      ValueTarget: FromStr,
{
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::Target::*;

        if let Ok(target) = s.parse::<FireTarget>() {
            Ok(Fire(target))
        } else if let Ok(target) = s.parse::<SwitchTarget>() {
            Ok(Switch(target))
        } else if let Ok(target) = s.parse::<ValueTarget>() {
            Ok(Value(target))
        } else {
            Err(format!("Unknown target '{}'!", s))
        }
    }
}

#[derive(Debug)]
pub enum ControlBind<FireTarget, SwitchTarget, ValueTarget> {
    Fire(FireTrigger, FireTarget),
    Switch(HoldableTrigger, SwitchTarget),
    Value(ValueTrigger, ValueTarget),
}

#[derive(Debug, Default)]
struct HoldableTriggerData<FireTarget, SwitchTarget>
where FireTarget: Eq + Hash,
      SwitchTarget: Eq + Hash,
{
    on_press: HashSet<FireTarget>,
    while_down: HashSet<SwitchTarget>,
    device_counters: HashMap<DeviceId, u32>,
    overall_counter: u32,
}

impl<FireTarget, SwitchTarget> HoldableTriggerData<FireTarget, SwitchTarget>
where FireTarget: Eq + Hash,
      SwitchTarget: Eq + Hash,
{
    fn new() -> Self {
        Self {
            on_press: HashSet::new(),
            while_down: HashSet::new(),
            device_counters: HashMap::new(),
            overall_counter: 0,
        }
    }
}

#[derive(Debug, Default)]
struct MouseWheelMapping<FireTarget, ValueTarget>
    where FireTarget: Eq + Hash,
          ValueTarget: Eq + Hash,
{
    on_up: HashSet<FireTarget>,
    on_down: HashSet<FireTarget>,
    on_change: HashSet<ValueTarget>,
}

impl<FireTarget, ValueTarget> MouseWheelMapping<FireTarget, ValueTarget>
where FireTarget: Eq + Hash,
      ValueTarget: Eq + Hash,
{
    fn new() -> Self {
        Self {
            on_up: HashSet::new(),
            on_down: HashSet::new(),
            on_change: HashSet::new(),
        }
    }
}

#[derive(Debug)]
pub enum ControlEvent<FireTarget, SwitchTarget, ValueTarget> {
    Fire(FireTarget),
    Switch { target: SwitchTarget, state: SwitchState },
    Value { target: ValueTarget, value: f64 },
}

pub struct Controls<FireTarget, SwitchTarget, ValueTarget>
where FireTarget: Eq + Hash,
      SwitchTarget: Eq + Hash,
      ValueTarget: Eq + Hash,
{
    holdable_trigger_data: HashMap<HoldableTrigger, HoldableTriggerData<FireTarget, SwitchTarget>>,
    axis_mappings: HashMap<u32, HashSet<ValueTarget>>,
    mouse_wheel_mapping: MouseWheelMapping<FireTarget, ValueTarget>,
    switch_counter: HashMap<SwitchTarget, u32>,
    value_factors: HashMap<ValueTarget, f64>,
    events: VecDeque<ControlEvent<FireTarget, SwitchTarget, ValueTarget>>,
}

impl<FireTarget, SwitchTarget, ValueTarget> Controls<FireTarget, SwitchTarget, ValueTarget>
where FireTarget: Copy + Eq + Hash + FromStr + ToString,
      SwitchTarget: Copy + Eq + Hash + FromStr + ToString,
      ValueTarget: ValueTargetTrait + Copy + Eq + Hash + FromStr + ToString,
{
    pub fn new() -> Self {
        Controls {
            holdable_trigger_data: HashMap::new(),
            axis_mappings: HashMap::new(),
            mouse_wheel_mapping: MouseWheelMapping::new(),
            switch_counter: HashMap::new(),
            value_factors: HashMap::new(),
            events: VecDeque::new()
        }
    }

    pub fn from_toml(value: &toml::value::Value) -> Result<Self, String> {
        use self::ControlBind::*;
        use toml::Value::Table;
        use toml::Value::Float;

        let mut controls = Controls::new();
        let table = match value {
            &Table(ref t) => t,
            _ => return Err(String::from("Controls must be a table!")),
        };

        match table.get("binds") {
            Some(v) => match v {
                &Table(ref keys) => for (target_string, trigger_value) in keys {
                    let bind = match target_string.parse()? {
                        Target::Fire(target) =>
                            Fire(FireTrigger::from_toml(trigger_value)?, target),
                        Target::Switch(target) =>
                            Switch(HoldableTrigger::from_toml(trigger_value)?, target),
                        Target::Value(target) =>
                            Value(ValueTrigger::from_toml(trigger_value)?, target),
                    };
                    controls.add_bind(bind);
                },
                _ => return Err(String::from("Binds must be a table!")),
            },
            None => return Err(String::from("No binds section found in controls!")),
        }
        match table.get("factors") {
            Some(v) => match v {
                &Table(ref factors) => for (target_string, trigger_value) in factors {
                    match target_string.parse::<Target<FireTarget, SwitchTarget, ValueTarget>>()? {
                        Target::Value(target) => match trigger_value {
                            &Float(factor) => controls.set_factor(target, factor),
                            v => return Err(format!("Factor must be a float, got '{}'!", v)),
                        }
                        _ => return Err(format!("Expected value target!")),
                    };
                },
                _ => return Err(String::from("Binds must be a table!")),
            },
            None => return Err(String::from("No binds section found in controls!")),
        }
        Ok(controls)
    }

    pub fn to_toml(&self) -> toml::value::Value {
        use self::FireTrigger::*;
        use self::ValueTrigger::*;
        use self::MouseWheelDirection::*;
        use toml::Value::Table;
        use toml::Value::Float;

        let mut binds = BTreeMap::new();
        for (&trigger, data) in self.holdable_trigger_data.iter() {
            for target in data.on_press.iter() {
                binds.insert(target.to_string(), Holdable(trigger).to_toml());
            }
            for target in data.while_down.iter() {
                binds.insert(target.to_string(), trigger.to_toml());
            }
        }
        for (&axis, mapping) in self.axis_mappings.iter() {
            for target in mapping {
                binds.insert(target.to_string(), toml::value::Value::Integer(axis as i64));
            }
        }
        for target in self.mouse_wheel_mapping.on_up.iter() {
            binds.insert(target.to_string(), MouseWheelTick(Up).to_toml());
        }
        for target in self.mouse_wheel_mapping.on_down.iter() {
            binds.insert(target.to_string(), MouseWheelTick(Down).to_toml());
        }
        for target in self.mouse_wheel_mapping.on_change.iter() {
            binds.insert(target.to_string(), MouseWheel.to_toml());
        }

        let mut factors = BTreeMap::new(); // TODO maybe just clone?
        for (target, &factor) in self.value_factors.iter() {
            factors.insert(target.to_string(), Float(factor));
        }
        Table(vec![
            (String::from("binds"), Table(binds)),
            (String::from("factors"), Table(factors)),
        ].into_iter().collect())
    }

    pub fn set_factor(&mut self, target: ValueTarget, factor: f64) {
        self.value_factors.insert(target, factor);
    }

    pub fn add_bind(&mut self, bind: ControlBind<FireTarget, SwitchTarget, ValueTarget>) {
        match bind {
            ControlBind::Fire(trigger, target) => self.add_fire_bind(trigger, target),
            ControlBind::Switch(trigger, target) => self.add_switch_bind(trigger, target),
            ControlBind::Value(trigger, target) => self.add_value_bind(trigger, target),
        };
    }

    pub fn remove_bind(&mut self, bind: ControlBind<FireTarget, SwitchTarget, ValueTarget>) {
        match bind {
            ControlBind::Fire(trigger, target) => self.remove_fire_bind(trigger, target),
            ControlBind::Switch(trigger, target) => self.remove_switch_bind(trigger, target),
            ControlBind::Value(trigger, target) => self.remove_value_bind(trigger, target),
        };
    }

    pub fn process(
        &mut self,
        device_id: DeviceId,
        device_event: DeviceEvent,
    ) {
        match device_event {
            DeviceEvent::MouseWheel { delta } => self.on_mouse_wheel(device_id, delta),
            DeviceEvent::Motion { axis, value } => self.on_motion(device_id, axis, value),
            DeviceEvent::MouseMotion { delta } => self.on_mouse_motion(device_id, delta),
            DeviceEvent::Button { button, state } => self.on_button(device_id, button, state),
            DeviceEvent::Key(input) => self.on_keyboard_input(device_id, input),
            DeviceEvent::Removed => self.on_device_removed(device_id),
            _ => (),
        }
    }

    pub fn get_events(&mut self) -> Drain<ControlEvent<FireTarget, SwitchTarget, ValueTarget>> {
        self.events.drain(..)
    }

    fn add_fire_bind(&mut self, trigger: FireTrigger, target: FireTarget) {
        use self::FireTrigger::*;
        use self::MouseWheelDirection::*;

        match trigger {
            Holdable(holdable_trigger) => {
                self.holdable_trigger_data.entry(holdable_trigger)
                    .or_insert_with(HoldableTriggerData::new)
                    .on_press.insert(target);
            },
            MouseWheelTick(direction) => {
                let mapping = &mut self.mouse_wheel_mapping;
                match direction {
                    Up => mapping.on_up.insert(target),
                    Down => mapping.on_down.insert(target),
                };
            }
        };
    }

    fn add_switch_bind(&mut self, trigger: HoldableTrigger, target: SwitchTarget) {
        let data = self.holdable_trigger_data.entry(trigger)
            .or_insert_with(HoldableTriggerData::new);
        let bind_is_new = data.while_down.insert(target);
        let trigger_is_active = data.overall_counter > 0;
        if bind_is_new && trigger_is_active {
            Self::increase_switch_target_counter(
                target,
                &mut self.switch_counter,
                &mut self.events
            );
        }
    }

    fn add_value_bind(&mut self, trigger: ValueTrigger, target: ValueTarget) {
        use self::ValueTrigger::*;

        match trigger {
            MouseX => {
                // TODO
            },
            MouseY => {
                // TODO
            },
            MouseWheel => {
                self.mouse_wheel_mapping.on_change.insert(target);
            },
            Axis(axis) => {
                self.axis_mappings.entry(axis).or_insert_with(Default::default).insert(target);
            },
        };
    }

    fn remove_fire_bind(&mut self, trigger: FireTrigger, target: FireTarget) {
        use self::FireTrigger::*;
        use self::MouseWheelDirection::*;

        match trigger {
            Holdable(holdable_trigger) => {
                self.holdable_trigger_data.get_mut(&holdable_trigger)
                    .map(|binding| binding.on_press.remove(&target));
            },
            MouseWheelTick(Up) => { self.mouse_wheel_mapping.on_up.remove(&target); },
            MouseWheelTick(Down) => { self.mouse_wheel_mapping.on_down.remove(&target); },
        }
    }

    fn remove_switch_bind(&mut self, trigger: HoldableTrigger, target: SwitchTarget) {
        if let Some(data) = self.holdable_trigger_data.get_mut(&trigger) {
            let bind_existed = data.while_down.remove(&target);
            let trigger_is_active = data.overall_counter > 0;
            if bind_existed && trigger_is_active {
                Self::decrease_switch_target_counter(
                    target,
                    &mut self.switch_counter,
                    &mut self.events
                );
            }
        }
    }

    fn remove_value_bind(&mut self, trigger: ValueTrigger, target: ValueTarget) {
        use self::ValueTrigger::*;

        match trigger {
            MouseX => {
                // TODO
            },
            MouseY => {
                // TODO
            },
            MouseWheel => {
                self.mouse_wheel_mapping.on_change.remove(&target);
            },
            Axis(axis) => {
                self.axis_mappings.get_mut(&axis).map(|binding| binding.remove(&target));
            },
        };
    }

    fn on_motion(&mut self, _device_id: DeviceId, axis: u32, mut value: f64) {
        use self::ControlEvent::*;

        if let Some(mapping) = self.axis_mappings.get(&axis) {
            for &target in mapping {
                let factor = self.value_factors.get(&target).unwrap_or(&1.0);
                if value != 0.0 {
                    value *= factor * target.base_factor();
                    self.events.push_back(Value { target, value });
                }
            }
        }
    }

    fn on_mouse_motion(&mut self, _device_id: DeviceId, _delta: (f64, f64)) {
        // TODO
        /*use self::ControlEvent::*;

        if let Some(mapping) = self.axis_mappings.get(&axis) {
            for &target in mapping {
                let factor = self.value_factors.get(&target).unwrap_or(&1.0);
                if value != 0.0 {
                    value *= factor * target.base_factor();
                    self.events.push_back(Value { target, value });
                }
            }
        }*/
    }

    fn on_keyboard_input(&mut self, device_id: DeviceId, input: KeyboardInput) {
        use self::HoldableTrigger::*;
        if let Some(key_code) = input.virtual_keycode {
            self.handle_holdable_trigger(KeyCode(key_code), device_id, input.state);
        }
        self.handle_holdable_trigger(ScanCode(input.scancode), device_id, input.state);
    }

    fn on_button(&mut self, device_id: DeviceId, button_id: ButtonId,
                 state: ElementState) {
        self.handle_holdable_trigger(HoldableTrigger::Button(button_id), device_id, state);
    }

    fn on_mouse_wheel(&mut self, _device_id: DeviceId, delta: MouseScrollDelta) {
        use self::MouseScrollDelta::*;
        use self::ControlEvent::*;

        let value = match delta { // TODO also handle x and PixelDelta?
            LineDelta(_x, y) => y as f64,
            PixelDelta(_) => return,
        };

        if value < 0.0 {
            for &fire_target in self.mouse_wheel_mapping.on_up.iter() {
                self.events.push_back(Fire(fire_target));
            }
        } else if value > 0.0 {
            for &fire_target in self.mouse_wheel_mapping.on_down.iter() {
                self.events.push_back(Fire(fire_target));
            }
        }
        for &target in self.mouse_wheel_mapping.on_change.iter() {
            self.events.push_back(Value { target, value });
        }
    }

    fn handle_holdable_trigger(&mut self, trigger: HoldableTrigger, device_id: DeviceId,
                               state: ElementState) {
        use self::ElementState::*;
        use self::ControlEvent::*;

        let data = self.holdable_trigger_data.entry(trigger)
            .or_insert_with(HoldableTriggerData::new);
        let device_counter = data.device_counters.entry(device_id).or_insert(0);
        let overall_counter = &mut data.overall_counter;
        match state {
            Pressed => {
                *device_counter += 1;
                *overall_counter += 1;
                if *overall_counter != 1 {
                    return;
                }
            },
            Released => {
                debug_assert!(
                    *device_counter > 0,
                    "Tried to decrease holdable trigger per-device counter that is {}",
                    *device_counter
                );
                debug_assert!(
                    *overall_counter > 0,
                    "Tried to decrease holdable trigger overall counter that is {}",
                    *overall_counter);
                *device_counter -= 1;
                *overall_counter -= 1;
                if *overall_counter != 0 {
                    return;
                }
            },
        }

        if let Some(data) = self.holdable_trigger_data.get_mut(&trigger) {
            if state == Pressed {
                for &fire_target in data.on_press.iter() {
                    self.events.push_back(Fire(fire_target));
                }
            }
            for &switch_target in data.while_down.iter() {
                match state {
                    Pressed => Self::increase_switch_target_counter(
                        switch_target,
                        &mut self.switch_counter,
                        &mut self.events
                    ),
                    Released => Self::decrease_switch_target_counter(
                        switch_target,
                        &mut self.switch_counter,
                        &mut self.events
                    ),
                }
            }
        }
    }

    fn on_device_removed(&mut self, device_id: DeviceId) {
        for data in self.holdable_trigger_data.values_mut() {
            if let Some(device_counter) = data.device_counters.get_mut(&device_id) {
                data.overall_counter -= *device_counter;
                *device_counter = 0;
            }
        }
    }

    fn increase_switch_target_counter(
        target: SwitchTarget,
        switch_counter: &mut HashMap<SwitchTarget, u32>,
        events: &mut VecDeque<ControlEvent<FireTarget, SwitchTarget, ValueTarget>>
    ) {
        let counter = switch_counter.entry(target).or_insert(0);
        if *counter == 0 {
            events.push_back(ControlEvent::Switch {
                target,
                state: SwitchState::Active,
            });
        }
        *counter += 1;
    }

    fn decrease_switch_target_counter(
        target: SwitchTarget,
        switch_counter: &mut HashMap<SwitchTarget, u32>,
        events: &mut VecDeque<ControlEvent<FireTarget, SwitchTarget, ValueTarget>>,
    ) {
        let counter = switch_counter.entry(target).or_insert(0);
        debug_assert!(*counter > 0, "Tried to decrease switch target counter that is {}", *counter);
        *counter -= 1;
        if *counter == 0 {
            events.push_back(ControlEvent::Switch {
                target,
                state: SwitchState::Inactive,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use winit::EventsLoop;
    use winit::Event;
    use winit::WindowEvent;
    use winit::Window;

    use strum_macros::EnumString;
    use strum_macros::ToString;

    use crate::Controls;
    use crate::ControlBind;
    use crate::ValueTargetTrait;
    use crate::FireTrigger;
    use crate::HoldableTrigger;
    use crate::ValueTrigger;
    use crate::MouseWheelDirection;
    use crate::VirtualKeyCode;

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, ToString, EnumString)]
    enum FireTarget {
        LMBFire,
        MWUpFire,
        MWDownFire,
        GHFire,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, ToString, EnumString)]
    enum SwitchTarget {
        RMBSwitch,
        GHSwitch,
        Key0Switch,
        AMMBSwitch,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, ToString, EnumString)]
    enum ValueTarget {
        MouseX,
    }

    impl ValueTargetTrait for ValueTarget {
        fn base_factor(&self) -> f64 {
            1.0
        }
    }

    #[test]
    fn test_all() {
        let mut events_loop = EventsLoop::new();
        let _window = Window::new(&events_loop).unwrap();
        let mut controls: Controls<FireTarget, SwitchTarget, ValueTarget> = Controls::new();
        controls.add_bind(ControlBind::Fire(FireTrigger::Holdable(HoldableTrigger::Button(1)), FireTarget::LMBFire));
        controls.add_bind(ControlBind::Fire(FireTrigger::Holdable(HoldableTrigger::Button(1)), FireTarget::LMBFire)); // double bind ;)
        controls.add_bind(ControlBind::Fire(FireTrigger::MouseWheelTick(MouseWheelDirection::Up), FireTarget::MWUpFire));
        controls.add_bind(ControlBind::Fire(FireTrigger::MouseWheelTick(MouseWheelDirection::Down), FireTarget::MWDownFire));
        controls.add_bind(ControlBind::Fire(FireTrigger::Holdable(HoldableTrigger::KeyCode(VirtualKeyCode::G)), FireTarget::GHFire));
        controls.add_bind(ControlBind::Fire(FireTrigger::Holdable(HoldableTrigger::KeyCode(VirtualKeyCode::H)), FireTarget::GHFire));

        controls.add_bind(ControlBind::Switch(HoldableTrigger::Button(3), SwitchTarget::RMBSwitch));
        controls.add_bind(ControlBind::Switch(HoldableTrigger::KeyCode(VirtualKeyCode::G), SwitchTarget::GHSwitch));
        controls.add_bind(ControlBind::Switch(HoldableTrigger::KeyCode(VirtualKeyCode::G), SwitchTarget::GHSwitch)); // double bind ;)
        controls.add_bind(ControlBind::Switch(HoldableTrigger::KeyCode(VirtualKeyCode::H), SwitchTarget::GHSwitch));
        controls.add_bind(ControlBind::Switch(HoldableTrigger::KeyCode(VirtualKeyCode::Key0), SwitchTarget::Key0Switch));
        controls.add_bind(ControlBind::Switch(HoldableTrigger::Button(2), SwitchTarget::AMMBSwitch));
        controls.add_bind(ControlBind::Switch(HoldableTrigger::KeyCode(VirtualKeyCode::A), SwitchTarget::AMMBSwitch));

        controls.add_bind(ControlBind::Value(ValueTrigger::Axis(0), ValueTarget::MouseX));

        let mut close_requested = false;
        while !close_requested {
            events_loop.poll_events(|event| {
                //eprintln!("{:?}", event);
                match event {
                    Event::DeviceEvent { device_id, event } => controls.process(device_id, event),
                    Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => close_requested = true,
                    _ => (),
                }
            });

            for event in controls.get_events() {
                eprintln!("{:?}", event);
            }
        }
    }
}