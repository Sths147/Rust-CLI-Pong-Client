#!/bin/bash

############################################################
# Help                                                     #
############################################################
Help()
{
   # Display Help
   echo "Modify the keyboard events and run the Pong client"
   echo "in a docker container"
   echo "      "
   echo "Syntax: ./launch.sh LOCATION [R_DELAY] [R_RECURSIVE]"
   echo "      "
   echo "R_DELAY is to custom delay before first KeyRepeat Event" 
   echo "(default 660 ms)"
   echo "R_RECURSIVE is to custom delay between KeyRepeat Events"
   echo "(default 25 ms)"
   echo "      "
   echo "options:"
   echo "-h     print this help."
   echo
}

############################################################
############################################################
# Main program                                             #
############################################################
############################################################

while getopts ":h" option; do
   case $option in
      h) # display Help
         Help
         exit;;
     \?) # Invalid option
         echo "Error: Invalid option"
         exit;;
   esac
done

r_delay=660
r_rate=25

if [ -n "$3" ]
then
    r_delay=$2
    r_rate=$3
fi

if [[ ! "$r_delay" =~ ^[0-9]+$ || ! "$r_rate" =~ ^[0-9]+$ ]]; then
    echo "Error: invalid arguments for R_DELAY or R_RATE arguments"
    exit 1
fi

xset r rate 150 30
docker run --rm --name cli-pong -it cli:latest ./cli_app $1
xset r rate $r_delay $r_rate
echo "Thank you for playing"