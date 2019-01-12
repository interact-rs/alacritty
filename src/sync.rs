// Copyright 2016 Joe Wilm, The Alacritty Project Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Synchronization types
//!
//! Most importantly, a fair mutex is included
use parking_lot::{Mutex, MutexGuard};

/// A fair mutex
///
/// Uses an extra lock to ensure that if one thread is waiting that it will get
/// the lock before a single thread can re-lock it.
pub struct FairMutex<T> {
    /// Data
    data: Mutex<T>,
    /// Next-to-access
    next: Mutex<()>,
}

impl<T> FairMutex<T> {
    /// Create a new fair mutex
    pub fn new(data: T) -> FairMutex<T> {
        FairMutex {
            data: Mutex::new(data),
            next: Mutex::new(()),
        }
    }

    /// Lock the mutex
    pub fn lock(&self) -> MutexGuard<'_, T> {
        // Must bind to a temporary or the lock will be freed before going
        // into data.lock()
        let _next = self.next.lock();
        self.data.lock()
    }
}

use interact::access::{Access, ReflectDirect};
use interact::climber::{ClimbError, Climber};
use interact::deser::{self, Tracker};
use interact::{Deser, NodeTree, Reflector};
use std::sync::Arc;

impl<T> ReflectDirect for FairMutex<T>
    where T: Access
{
    fn immut_reflector(&self, reflector: &Arc<Reflector>) -> NodeTree {
        let locked = self.lock();
        Reflector::reflect(reflector, &*locked)
    }

    fn immut_climber<'a>(&self, climber: &mut Climber<'a>) -> Result<Option<NodeTree>, ClimbError> {
        let save = climber.clone();
        let retval = {
            let locked = self.lock();
            climber.general_access_immut(&*locked).map(Some)
        };

        if let Err(ClimbError::NeedMutPath) = &retval {
            *climber = save;
            let mut locked = self.lock();
            climber.general_access_mut(&mut *locked).map(Some)
        } else {
            retval
        }
    }

    fn mut_climber<'a>(&mut self, climber: &mut Climber<'a>) -> Result<Option<NodeTree>, ClimbError> {
        let mut locked = self.lock();
        climber.general_access_mut(&mut *locked).map(Some)
    }
}

impl<T> Deser for FairMutex<T>
where
    T: Deser,
{
    fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> deser::Result<Self> {
        Ok(FairMutex::new(T::deser(tracker)?))
    }
}

use interact::derive_interact_extern_opqaue;

derive_interact_extern_opqaue! {
    struct FairMutex<T>;
}
