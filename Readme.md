# Cli-app
### This application is designed to run independently of the server project
running this tool enables you to connect to the Pong website and play Pong on your terminal
## Warning: 
### Running the app modifies the delay of appearance of the Repeat Event of your keyboard, so close your app when you're done playing
Indeed, in order to improve playability of the game, we modify the settings of the keyboard events while the game is played. 
be sure to quit the game once you are done. The settings restored back are: 660ms delay for KeyRepeat and 25ms delay between repeats
you can adjust those settings with the command

```bash
xset r rate [firstRepeatDelay] [betweenRepeatDelay]
```

users using Wayland won't be impacted by these settings

## How to play:

### First time: Execute:

```bash
./build.sh
``` 
to build the docker image 

### Then execute:

```bash
./launch.sh $LOCATION
```
while replacing $LOCATION with the IP:PORT the transcendence server listens to 

Note: This application runs on a Docker container, so if the game server runs on the same machine as the cli app, running
```bash
./launch localhost:XXXX
```
will not work. You can replace IP with the docker0 bridge address or your IP address. You can get required information with the following command:
```bash
ip address
``` 