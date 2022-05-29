# DU_Analog_Inputs
## Disclaimer
You can map Xbox Gamepad buttons directly ingame and probably other devices as well. I learned this after developing the whole key mapping part. Touché NQ.

## Intro
This is a Rust tool that allows sending **several** analog inputs to the game Dual Universe (DU) by encoding them into relative mouse movements.
For usage, at least read the "Usage". If you want to know how it works and possibly improve it, see "How does the input get into the game?".
For any questions, feel free to hit me up on Discord via PM (ZarTaen#6409) or via the OSIN Discord Server https://discord.gg/B5fVQ7YKNM

## Working
- Xbox Gamepad all Analog Axis for 5Axis and 6Axis (See Lua Script Snippets and examples)
- Basically everything else should work to a degree (Aside from whatever JoyBallMotion is), but the default is for Xbox Gamepad and tested as such

## Usage
First, every Lua script that is supposed to work with this has to be adapted. The files under "Lua Script Snippets" demonstrate how I built the support into my ship for testing purposes.
There is an autoconfig based on a slightly improved default script, thanks to Davemane42.

The tool can be run whenever and looks for various game input devices, such as an Xbox Gamepad. Once the tool is running, it will have a small terminal window open. Don't be scared by that and keep it open, closing it will close the tool. The first output should be version number.
The second outputs should be your recognized devices and their GUID. Copy those GUIDS and note which device it is. They are needed to assign mappings. At first startup, the default configs will be created.
Use this chance to check for inputs, debug will be active. While debug is active, the Inputs will be printed into the terminal. But be prepared that an Xbox Gamepad is plug and play and will already use the default mappings. 

`^` or `Right Ctrl` will toggle encoding and send analog input. If this is active, your analog inputs **WILL** move your mouse cursor very violently, including camera movements, towards the bottom right by default.
It is intended to lock the camera ingame before activating the analog input with `^` or `Right Ctrl`. However analog input by default has this mapped to the start button for an Xbox Controller. In the Lua example, the additional key is the boost activation key.

For details about mapping a device, see "Mapping Files Explained". If a gamepad-like device isnt properly recognized, get the gamecontrollerdb.txt and place it next to the .exe. For some devices this may help.

---

Should the inputs not work as expected after setting everything up correctly, dont hesitate to tell me and when I have time, we can troubleshoot it.

## Mapping Files Explained
Axis Example with Xbox Gamepad GUID:
```toml
[mapping.030000005e040000ff02000000007200]
0 = 'XAxis1'
1 = 'YAxis1'
2 = 'XAxis2'
5 = 'ZAxis1'
4 = 'ZAxis2'
3 = 'YAxis2'
```
The first line of each toml block always has to be `[mapping.x]`, where x stands for your device GUID that is always printed at startup or when it is connected. You can have several of those blocks, for different ids, but not the same id! The left side is the axis ID of your device. This can be found when using and pressing them in debug mode.
The right side of the axis IDs are the input targets. For the `axis_mapping.toml` file, those have to be one of the Axis, as found in `/examples/axis_mapping_targets.txt`.
The same things as the Axis file apply to the `key_mapping.toml`, however, instead of Axis to map to, it will accept a comma separated key code list. 

KeyMap Example with Xbox Gamepad GUID:
```toml
[mapping.030000005e040000ff02000000007200]
7 = '220,66'
11 = '84'
6 = '220,9'
10 = '82'
5 = '18'
2 = '88'
4 = '17'
```
This means that you can map a single button to key combinations such as `alt+j`! Or the whole keyboard, whatever floats your boat.
The default key mappings are the most basic thing for Xbox Controllers: 
- `Start` (7) for toggling the Analog Input, by pressing `^` and `B` (220,66) (Change to assigned boost button in DU, I use `^` as boost and just removed `B`).
- `Select`? (6) for toggling the Analog Input when trying to tab, by pressing `^` + Tab in DU.
- `Left Bumper` (4) is Ctrl, for default brake
- `Right Bumper` (5) is Alt, for default modifier
- `DPad Up` (10) is R for Throttle Up
- `DPad Down` (11) is T for Throttle down
- `X` is X for Trajectory

I really recommend for the community to create mappings for various devices and rely on each other, as it is essentially impossible to do mappings for devices I do not own,
and it is all preference anyway.
After deliberating a long time, that is also why I decided not to include a fully key-mapped Xbox Gamepad.

For Keycodes see https://boostrobotics.eu/windows-key-codes/.
Beware that some keycodes will not be key specific (Ctrl (163) is changed to Ctrl (17/162) only for example)

## How does the input get into the game?
Every axis of a device is mapped to deliver a value between -1.0 and 1.0, other than Analog Triggers, those are 0.0 to 1.0.

Relative Mouse movement on Windows uses a signed 32 bit Integer value. This means, the maximum limit of relative mouse movement on windows is
2147483647 Pixels at once. As a result, there are theoretically 31 bits available for encoding, minus 1 for the canary(explained in a minute).

First, the axis have to be translated from float to a fitting integer range. By default, this is 0-63 for one side and 0-63 for another, for 6Axis.
Axis that are exclusive (such as Left Stick X Axis, with left and right not possible at the same time), share that space, by doing the following:

- Left Stick X-Axis 0 to -1.0 becomes 0 to 63 (left)
- Left Stick X-Axis 0 to 1.0 becomes 64 to 127 (right)

Once this is done with all axis, they are added to one final mouse movement with bitshifting. The following is going to have bit representations for the maximum intended possible value:

Mouse X-Axis for 6Axis:
* Canary bit at 2097152: 
  * `1`0 0000 0000 0000 0000 0000
* ZAxis1: 
  * 10 0000 0000 0000 0`111 1111`
* XAxis2: 
  * 1`1 1111 11`00 0000 0111 1111
* XAxis1: 
  * 11 1111 11`11 1111 1`111 1111

Mouse Y-Axis for 6Axis:
* Canary bit at 2097152: 
  * `1`0 0000 0000 0000 0000 0000
* ZAxis2: 
  * 10 0000 0000 0000 0`111 1111`
* YAxis2: 
  * 1`1 1111 11`00 0000 0111 1111
* YAxis1: 
  * 11 1111 11`11 1111 1`111 1111

5Axis is slightly different, it uses bigger ranges for both XAxis and YAxis, at 0-127, uses 0.0-1.0 axis of triggers, and has overall 1 more bit.

Mouse X-Axis for 5Axis:
* Canary bit at 4194304: 
  * `1`00 0000 0000 0000 0000 0000
* LesserAxis1: 
  * 10 0000 0000 0000 00`11 1111`
* XAxis2: 
  * 1`11 1111 11`00 0000 0011 1111
* XAxis1: 
  * 111 1111 11`11 1111 11`11 1111

Mouse Y-Axis for 5Axis:
* Canary bit at 4194304: 
  * `1`00 0000 0000 0000 0000 0000
* LesserAxis2: 
  * 10 0000 0000 0000 00`11 1111`
* YAxis2: 
  * 1`11 1111 11`00 0000 0011 1111
* YAxis1: 
  * 111 1111 11`11 1111 11`11 1111

Now, everyone that paid attention closely, realises that these are not 31 bits. There is a good reason for that. The game is highly inconsistent when it comes to reading mouse inputs. Sometimes, it reads 0, and sometimes it doubles, triples or even quadruples the mouse input in a single reading.
This by itself is not a problem. But the game also uses a 32 bit float to store the originally 32 bit integer input of the mouse. Floating points at higher values become highly inaccurate for our usecase. Anything above a certain amount of bits will end up in a loss of bits.
Therefore, values that are a multiples of the intended value are dropped ingame, so are 0s, and we use the canary. The canary allows us to check whether the whole value has multiplied.

Now, with the values encoded and a mouse input on the way, we are still not finished. While it is possible to ignore the multiplied and 0d inputs, if those become too many, the input will start feeling sluggish.
This is why only the input state is transmitted. It allows to reduce the transmission rate considerably while still being fairly responsive. Basically, the game thinks your input is the old state, until it is updated.
Something to mention: I tried simply dividing the multiplied inputs, but man oh man was it unpredictable as hell. I even saw .5s where none should be.

**The send rate limit is where I need as much feedback as possible, to find a most commonly playable value.**
Filtering 0.0s is straightforward: Every 0 input on our side is at least as big as the canary bit.
Multiples are always at least a multiple of the canary bit. So, it is straightforward.

Once the invalid mouse inputs are filtered, the whole float has to be converted to a bit string. There is a provided function in the Lua code snippets. The resulting bit string should be identical to the bit string we originally sent.
With this in mind, all we need to do is to take the ranges as shown above, and take out the respective string slices. Then convert those to integer values, do the whole encoding process basically in reverse.
At the end, the inputs end up as -1.0 to 0.0 to 1.0 again.

## Working with Lua
The code for unit.start and system.flush are necessary. You need the code snippets for: 
- Receiving mouse input and saving the value. (unit.start and system.flush)
- Filtering the mouse input and assigning the input axis (unit.start and system.update)
- Assigning the input axis to the ship controls (system.flush)
- Switching between classic controls and analog input (system.actionStart(YourChoice))

For something like Twinsticks, I recommend using 6Axis only. The provided Gamepad6Axis should in theory be working fine. Anything with more than 6 Axis will however not be possible without serious input resolution issues in the future.

Thanks to Davemane42 for doing the autoconfig based on the flying default. I modified it slightly to include more inverts and also more generic naming. It is a drop-in replacement for the NQ default flying script.

## Todo
- Gathering feedback for the send rate
- User-Friendliness

## Change in latest Version
- Changed input backend to SDL2 for better device compabitility.
- Changed analog input toggle key to `^` (220) and `Right Ctrl` (163)

## For Developers and Self-Compile Fans
This project uses the VCPKG approach under "Windows, Linux and macOS with vcpkg" to use SDL2 without .dll. Find more information under:
https://github.com/Rust-SDL2/rust-sdl2

## I want to contribute!
Feel free to do so. Especially for the Lua side of things, help is very appreciated. Do not hesitate to contact me either via Discord (ZarTaen#6409)
or over Github. You can also contribute by simply trying it out, verifying the device works in Windows in general but is not recognized by the used library, etc. Once I can confirm that a specific device is not working as intended, I can work on it.

A BIG THANKS TO Blazemonger and Davemane42 for help with testing!

## This is too complicated, I want to complain!
I too like to complain in my spare time. For complaints please open an issue or add to an existing one. If it is not solvable by me, pester NQ with "Analog Inputs When".