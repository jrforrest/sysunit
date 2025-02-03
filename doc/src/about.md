# About Sysunit

Sysunit is currently a solo side project by [Jack Forrest](https://jackforrest.me).

It was born out of frustration with the unsuitability of more complex tools for
simpler configuration tasks like setting up a new server or configuring environments
more minimal than Ansible can target, and a desire for something more robust than
the ad-hoc shell scripts I kept writing to do these jobs.

It started out as a shell script that just provided params to and parsed output
from my other scripts. It was re-written in Python in 2020, and then again in
Rust in 2022. In late 2024 some friends started using it, but I was pretty
embarrassed by the throwaway code and some architectural flaws, so I re-wrote
it again in async Rust and wrote some actual docs.

Now, in 2025, I think the shell-based paradigm has proven itself to myself
and a few others, and the code-base has reached enough architectural stability
that I figure it's ready for a wider audience.

This tool is shared in the hacker spirit, and does not aim to deliver a regimented
approach to configuration management. It affords a lot of flexibility and direct
control without faffing about with many abstractions, and its accepted that
can mean footguns for some potential users. However, I'm always looking for
better ways to do things if it doesn't mean trading too much power! Open
a [GitHub Issue](https://github.com/jrforrest/sysunit) with any feedback
or recommendations.
