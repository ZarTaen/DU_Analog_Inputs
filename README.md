# DU_Analog_Inputs (WIP)
## Intro
This is a small rust tool that allows sending **several** analog inputs to the game Dual Universe by encoding them into relative mouse movements.
For Usage, at least read the "How to use this". If you want to know how it works and possibly improve it, see "How does the input get into the game?".
For any questions, feel free to hit me up on Discord via PM (ZarTaen#6409) or via the OSIN Discord Server https://discord.gg/B5fVQ7YKNM

## Working
- Xbox Gamepad all Analog Axis

## What to know before using it
First, every Lua script that is supposed to work with this, has to be adapted. The files under "Lua Script Snippets" demonstrate how I built the support into my ship for testing purposes.
There are most likely better ways and formulas, as well as preferences on a player by player basis, but I will leave this part mostly to the community.
However, lets just say that the setup that I include as an example was adjusted by me to be as consistent as possible, framedrop or not. It is not perfect and feedback is appreciated.

The tool can be run whenever and looks for various game input devices, such as an XBOX Gamepad. 
The result is saved into the "device_list.txt". 
If you use Input Devices of various kinds that are not supported yet, please run the tool once anyway and send me this file on Discord or the data inside here in a Github issue.
Only then can I create mappings and further develop this tool for support.

---

Once the tool is running, it will have a small terminal window open. Don't be scared by that and keep it open, I simply did not put in the work to make it a tray icon instead.
If you want, I can get rid of it sometime. Your right CTRL key will toggle encoding and sending analog input.
You will notice it is active if your mouse hops at the spot or your camera wiggles inside the game. 

If this is active, your analog inputs **WILL** move your mouse cursor very violently, including camera movements. Therefore, a key ingame is needed to activate and deactivate the camera lock!
It is intended to lock the camera ingame before activating the analog input with the right CTRL key. 

---

If the input generally works, but sometimes does not, do not hesitate to tell me. This most likely means the send rate is still too high. I will make the send rate adjustable down the line.

## How does the input get into the game?
Every axis of a device is mapped to deliver a value between -1.0 and 1.0.
Relative Mouse movement on Windows uses a signed 32 bit Integer value. This means, the maximum limit of relative mouse movement on windows is
2147483647 Pixels at once. As a result, there are 10 digits that can be used for encoding, for one mouse delta direction.

Therefore, the current encoding for an Xbox Gamepad works as follows:

Mouse X-Axis:
- Left Stick X-Axis 0 to -1.0 becomes 0 to 400
- Left Stick X-Axis 0 to 1.0 becomes 401 to 800
- Right Stick X-Axis 0 to -1.0 becomes 000xxx to 400xxx
- Right Stick X-Axis 0 to 1.0 becomes 401xxx to 800xxx
- Right Trigger Axis from 0 to 1.0 becomes 0xxxxxx to 40xxxxxx

Mouse Y-Axis:
- Left Stick Y-Axis 0 to -1.0 becomes 0 to 400
- Left Stick Y-Axis 0 to 1.0 becomes 401 to 800
- Right Stick Y-Axis 0 to -1.0 becomes 000xxx to 400xxx
- Right Stick Y-Axis 0 to 1.0 becomes 401xxx to 800xxx
- Left Trigger Axis from 0 to 1.0 becomes 0xxxxxx to 40xxxxxx

With this, the values are encoded. Now, the question is: why just 8 digits and not all 10? Well, an additional hurdle is that Dual Universe uses float for Mouse Delta for some god forsaken reason.
This means that at high values in our range, the accuracy of the Left Stick would decrease, by not allowing 0, 1, 2, 3, 4 and so on, but rather only 0,8,16,24,32..
As a consequence, there is a tradeoff for Trigger Accuracy and Left Stick Accuracy in the current mapping.

Okay, so the value is in the game, and everything can just be decoded, right? WRONG.
The game is funny in that way. In order to have the best possible accuracy, its a good idea to transmit state, not changes.
However, the game drops and doubles mouse inputs fairly unpredictably, based on the rate the inputs are sent to the game.
This means, an input sequence for Mouse X-Axis can look like this:

- 0.0
- 20401`762`
- 4080`3524`
- 0.0
- 20401`762`
- 0.0

The highlighted numbers show the values affected by the Left Stick in case of a doubled value.

Therefore, 2 things had to be done:
- Reduce the transmission limit to the game to get rid of doubles
  - Doubled Values can **not** be corrected and results in an input such as Right Stick suddenly being decoded as Trigger Input as well!
- Filter out the dropped Mouse Values

**The send rate limit is where I need as much feedback as possible, to find a playable value.**
Doubled Inputs simply can not be corrected unless filtered by using averages of inputs. It is not made better with the fact that the rate WILL vary from player to player, possibly based on framerate even.

Filtering 0.0s is fairly straightforward. A jitter of 1 Pixel for both Mouse Axis is applied everytime the tool sends an input, even if the input by the gamepad is 0.
With deadzones, this is filtered out, and the game can filter out 0.0s as dropped mouse inputs as well.

## Todo
- Gathering feedback for the send rate
- Collecting Outputs for different game devices, such as HOTAS and co.
  - Consequently, potentially adapting the input library currently in use for broader support
- Creating Mappings for other input devices, based on the collected Outputs
- Input device to keyboard mapping
- More customizability
- Make sendrate and pollrate adjustable

## I want to contribute!
Feel free to do so. Especially for the Lua side of things, help is very appreciated. Do not hesitate to contact me either via Discord (ZarTaen#6409)
or over Github.

## This is too complicated, I want to complain!
I too like to complain in my spare time. For complaints please open an issue or add to an existing one. If it is not solvable by me, pester NQ with "Analog Inputs When".