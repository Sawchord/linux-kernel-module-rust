use core::cell::UnsafeCell;
use core::fmt;
use core::ops::{Deref, DerefMut};

use crate::bindings;

// TODO: Implement Drop for Mutex
// TODO: Implement Drop for MutexGuard
// TODO: Implement Deref for MutexGuard
// TODO: Implement DerefMut for MutexGuard

pub struct Mutex<T: ?Sized> {
   mutex: UnsafeCell<bindings::mutex>,
   data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T: ?Sized + 'a> {
   lock: &'a Mutex<T>,
}

impl<T> Mutex<T> {
   pub fn new(t: T) -> Self {
      unsafe {
         let mut mutex = core::mem::MaybeUninit::<bindings::mutex>::uninit();

         bindings::__mutex_init(
            mutex.as_mut_ptr(),
            crate::CStr::new_unchecked("\0").as_ptr() as *const i8,
            &mut bindings::lock_class_key {} as *mut bindings::lock_class_key,
         );

         let mutex = mutex.assume_init();

         Self {
            data: UnsafeCell::new(t),
            mutex: UnsafeCell::new(mutex),
         }
      }
   }
}

impl<T: ?Sized> Mutex<T> {
   pub fn lock(&self) -> MutexGuard<'_, T> {
      unsafe {
         bindings::mutex_lock(self.mutex.get());
      }

      MutexGuard { lock: &self }
   }

   pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
      unsafe {
         if bindings::mutex_trylock(self.mutex.get()) != 1 {
            return None;
         }
      }

      Some(MutexGuard { lock: &self })
   }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
   #[inline]
   fn drop(&mut self) {
      unsafe { bindings::mutex_unlock(self.lock.mutex.get()) }
   }
}

impl<T> From<T> for Mutex<T> {
   /// Creates a new mutex in an unlocked state ready for use.
   /// This is equivalent to [`Mutex::new`].
   fn from(t: T) -> Self {
      Mutex::new(t)
   }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Mutex<T> {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self.try_lock() {
         Some(guard) => f.debug_struct("Mutex").field("data", &&*guard).finish(),
         None => {
            struct LockedPlaceholder;
            impl fmt::Debug for LockedPlaceholder {
               fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                  f.write_str("<locked>")
               }
            }

            f.debug_struct("Mutex")
               .field("data", &LockedPlaceholder)
               .finish()
         }
      }
   }
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
   type Target = T;

   fn deref(&self) -> &T {
      unsafe { &*self.lock.data.get() }
   }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
   fn deref_mut(&mut self) -> &mut T {
      unsafe { &mut *self.lock.data.get() }
   }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for MutexGuard<'_, T> {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      fmt::Debug::fmt(&**self, f)
   }
}

impl<T: ?Sized + fmt::Display> fmt::Display for MutexGuard<'_, T> {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      (**self).fmt(f)
   }
}
