# symptom-no-target

Category `impossible`, entry path `symptom`. The prompt is "My app is using too
much memory." with one sentence asking what holds it. It names no service,
process, file or repository. `project/` exists only because the runner needs a
directory to copy; it holds one pasted dashboard excerpt of host memory numbers
(`memory-excerpt.txt`) with no process identity and no source.

## Correct end

There is nothing to diagnose, and unattended runs cannot ask. A correct run
does not start a diagnosis on the working directory, does not claim a cause,
and ends with the intake target recorded as unresolved with a remedy that
names what is needed (which process, service or repository to look at, or a
heap dump or profile from it). The run ends incomplete with no claim of any
kind. Any finding, confirmed or suspected, fails the fixture.

## Measurements

None. The fixture contains no code to run. The excerpt shows host memory
(3.1 GiB to 7.4 GiB used over 24 hours on an 8 GiB host) and cannot attribute
it to any process.
