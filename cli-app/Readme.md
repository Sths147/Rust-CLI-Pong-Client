# Cli-app
### This application is designed to run independently of the server project
running this tool enables you to connect to the Pong's website and play Pong on your terminal
## Warning: 
### While running the app modifies the delay of appearance of the Repeat Event of your keyboard, so close your app when you're done playing
Indeed, in order to improve playability of the game, we modify the settings of the keyboard events while the game is played. 
be sure to leave the game once you are done. The settings set back are: 500ms delay for KeyRepeat and 30ms delay between repeats
you can adjust those settings with the `xset r rate [firstRepeatDelay] [betweenRepeatDelay]` bash command
users using wayland won't be impacted by these settings

## How to play:

### First time: Execute:

`./build.sh` in your console

### Then execute:

`./launch.sh $LOCATION` while replacing $LOCATION with the IP:PORT the transcendance servers listens to 

Note: This application runs on a Docker container, so if the game server runs on the same machine as the cli app, running `./launch localhost:XXXX` will not work. You can replace IP with the docker0 bridge address, usually 172.17.0.1.