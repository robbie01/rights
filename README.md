# Exploring SCM_RIGHTS

This is an experiment in enabling [privilege separation](https://en.wikipedia.org/wiki/Privilege_separation) in Rust with Tokio. I develop mainly on my MacBook Air M2 these days, so this experiment primarily concerns macOS.

- `sendmsg` and `recvmsg` MUST have at least one `IoSlice`/`IoSliceMut` on macOS or they will yield `EMSGSIZE`. This is not true of FreeBSD, which is okay with having zero `IoSlice`s/`IoSliceMut`s.
- `recvmsg` SHOULD be attached to a buffer of nonzero length, even if no data is ever sent. Otherwise, the socket will continue to be armed and it will loop infinitely. Again, this does not seem to be true of FreeBSD.
- For some reason, some of the messages received are blank, meaning that the received fd yielded no bytes before going EOF. This does not appear to happen on FreeBSD. ~~I can't figure this one out.~~ Notice the now-commented line reading `drop(there)`. There appears to be some sort of race condition at play here, which is fixed (rather cheesily) by effectively delaying its drop by a second. A more robust solution could confirm receipt of the fd somehow, then drop it.

[libev's diatribe](https://pod.tst.eu/http://cvs.schmorp.de/libev/ev.pod#OS_X_AND_DARWIN_BUGS) is evergreen, I suppose.