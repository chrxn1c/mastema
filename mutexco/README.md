# Mutex & Company

A simple implementation of threaded Mutex, which is backed up by `atomic-wait` crate, allowing to omit the platform implementation
of the underlying mechanism used (be it `futex` syscall on Linux, OSX-specific wake & wait API, or Windows-specific wake & wait API).

Also, there is a Condition Variable implementation. Condition Variable is used to together with a mutex to wait until the mutex-protected data matches some condition.

> Note: All platforms are supported

