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



if [[ "${release}" == "centos" ]]; then
	echo -e " ${green} You use centos \n" 

elif [[ "${release}" == "ubuntu" ]]; then
	echo -e "${green} You use ubuntu \n"
else 
       echo -e " ${red} Failed to check Os. Your Os System not Supported " && exit 1
fi

install-ubuntu() {
apt update  -y
apt install bind9 bind9utils git bind9-doc -y 
}

install-centos(){
yum update -y
yum install bind bind-utils git -y
}


install-depend(){
case ${release} in 
	centos)
		install-centos
		;;
	ubuntu) 
		install-ubuntu
		;;
	*) 
		echo -e " Invalid choice. Please try again."
		;;
esac
}

install-depend






















