# Notepad Hijacker

This is an experiment to read/write to the memory of a process in Windows.
It turns out that the Windows API lets you access other processes pretty
easily!

Currently, we are able to:

* Detect whether notepad.exe is running
* Extract the text that is being edited

Possible future experiments are:
* Change the text in the editor by something of our own choice

Note: the `0x2C470` offset used in the code seems to be machine-dependent,
so if you run this code on your computer it will probably trigger an error.
You _could_ reverse engineer your notepad version to get a similar offset,
though (but I don't have the time now to explain how).

# License

MIT
