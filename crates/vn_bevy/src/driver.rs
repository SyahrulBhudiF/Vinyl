use bevy::prelude::*;
use soa_rs::Soa;
use vn_core::{PresentationSnapshot, Program, RollbackCheckpoint, Vm, VmError, VmEvent, VmState};

/// Owns the deterministic story VM inside Bevy.
#[derive(Resource)]
pub struct VnStory {
    vm: Vm,
    last_event: Option<VmEvent>,
    last_error: Option<VmError>,
    revision: u64,
}

impl VnStory {
    /// Creates a story driver from a compiled program.
    pub fn new(program: Program) -> Result<Self, VmError> {
        Ok(Self {
            vm: Vm::new(program)?,
            last_event: None,
            last_error: None,
            revision: 0,
        })
    }

    /// Restores a localized story driver from a save.
    pub fn from_parts(
        program: Program,
        state: VmState,
        presentation: PresentationSnapshot,
        rollback: Soa<RollbackCheckpoint>,
        translations: std::collections::HashMap<String, String>,
    ) -> Self {
        let mut vm = Vm::from_parts(program, state, presentation, rollback);
        vm.set_translations(translations);
        Self {
            vm,
            last_event: None,
            last_error: None,
            revision: 0,
        }
    }

    /// Creates a localized story driver.
    pub fn with_translations(
        program: Program,
        translations: std::collections::HashMap<String, String>,
    ) -> Result<Self, VmError> {
        Ok(Self {
            vm: Vm::with_translations(program, translations)?,
            last_event: None,
            last_error: None,
            revision: 0,
        })
    }

    /// Returns the underlying VM for save serialization.
    pub fn vm(&self) -> &Vm {
        &self.vm
    }

    /// Returns the latest successful VM event.
    pub fn last_event(&self) -> Option<&VmEvent> {
        self.last_event.as_ref()
    }

    /// Returns the latest VM error, if any.
    pub fn last_error(&self) -> Option<&VmError> {
        self.last_error.as_ref()
    }

    /// Returns the monotonic story-state revision used by persistence coordination.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns whether the latest successful event ended the story.
    pub fn ended(&self) -> bool {
        matches!(self.last_event, Some(VmEvent::End))
    }

    /// Advances until the next interaction event.
    pub fn continue_story(&mut self) -> Result<VmEvent, VmError> {
        match self.vm.continue_until_interaction() {
            Ok(event) => {
                self.record(event.clone());
                Ok(event)
            }
            Err(error) => {
                let stored = error.clone();
                self.last_error = Some(error);
                Err(stored)
            }
        }
    }

    /// Replaces this story with restored save state.
    pub fn restore(
        &mut self,
        program: Program,
        state: VmState,
        presentation: PresentationSnapshot,
        rollback: Soa<RollbackCheckpoint>,
        translations: std::collections::HashMap<String, String>,
    ) {
        let revision = self.revision.wrapping_add(1);
        *self = Self::from_parts(program, state, presentation, rollback, translations);
        self.revision = revision;
    }

    /// Restores the previous interaction checkpoint.
    pub fn rollback(&mut self) -> Option<VmEvent> {
        let event = self.vm.rollback()?;
        self.record(event.clone());
        Some(event)
    }

    /// Chooses a zero-based menu option.
    pub fn choose(&mut self, choice: usize) -> Result<VmEvent, VmError> {
        match self.vm.choose(choice) {
            Ok(event) => {
                self.record(event.clone());
                Ok(event)
            }
            Err(error) => {
                let stored = error.clone();
                self.last_error = Some(error);
                Err(stored)
            }
        }
    }

    fn record(&mut self, event: VmEvent) {
        self.last_event = Some(event);
        self.last_error = None;
        self.revision = self.revision.wrapping_add(1);
    }
}
