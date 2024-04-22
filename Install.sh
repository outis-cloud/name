#!/bin/bash

sleep 1
clear 
echo " Started ..." 
sleep 2 


#colors 


red='\033[0;31m'
green='\033[0;32m'
yellow='\033[0;33m'
blue='\033[0;34m'
purple='\033[0;35m'
cyan='\033[0;36m'
white='\033[0;37m'
rest='\033[0m'
plain='\033[0m'




#check root 

[[ $EUID -ne 0 ]] && echo "${red} Error Error :${plain} Please run this script with root \n " && exit 1


# check OS 

if [[ -f /etc/os-release ]]; then 
	source /etc/os-release
	release=$ID
elif [[ -f /usr/lib/os-release ]]; then 
	source /usr/lib/os-release
	releass=$ID
else 
	echo "Failed to check the system OS " >&2
	exit 1
fi


echo "The OS release is  $release "





















