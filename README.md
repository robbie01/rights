# Exploring SCM_RIGHTS

This is an experiment in enabling [privilege separation](https://en.wikipedia.org/wiki/Privilege_separation) in Rust with Tokio. I develop mainly on my MacBook Air M2 these days, so this experiment primarily concerns macOS.

- `sendmsg` and `recvmsg` MUST have at least one `IoSlice`/`IoSliceMut` on macOS or they will yield `EMSGSIZE`. This is not true of FreeBSD, which is okay with having zero `IoSlice`s/`IoSliceMut`s.
- `recvmsg` SHOULD be attached to a buffer of nonzero length, even if you do not intend to receive any data. Otherwise, the socket will not leave the readable state and it will loop infinitely. Again, this does not seem to be true of FreeBSD.
- For some reason, some of the messages received are blank, meaning that the received fd yielded no bytes before going EOF. This does not appear to happen on FreeBSD. I can't figure this one out.