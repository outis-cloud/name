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



ubuntu() {
apt update  -y
apt install bind9 bind9utils git bind9-doc lolcat figlet  -y 
nameserver

}

centos() {
yum update -y
yum install bind bind-utils git -y
}


if [[ "${release}" == "centos" ]]; then
         echo "vvvvvvvvvv"
elif [[ "${release}" == "ubuntu" ]]; then
           ubuntu
else
echo -e " ${red} Failed to check Os. Your Os System not Supported " && exit 1
fi



install_dep() {
case "${release}" in 
	centos)
		centos
		;;
	ubuntu) 
		ubuntu
		nameserver
		;;
	*) 
		echo " Failed to check the OS version"
		exit 1;;
esac
}


sleep 1

nameserver() { 

read -p "${yellow} Please Enter Domains Name:" $domain
read -p "${yellow} Please Enter Your Ip server:" $ip

if [ -z  "$domain" ] && [ -z "$ip" ]; then
	echo "Please A domain Real And Ip server "
else
	filedb
	changenameserver
	finish
fi
}

filedb() {
	cd /etc/bind/
	touch $domain.db
	cat <<EOL > $domain.db
	$TTL 3600
@       IN      SOA     ns1.$domain.      hostmaster.$domain. (
                                                2024042710
                                                3600
                                                3600
                                                1209600
                                                86400 )

$domain.      3600    IN      NS      ns1.$domain.
$domain.      3600    IN      NS      ns2.$domain.

$domain.      3600    IN      A       $ip
ftp     3600    IN      A       $ip
mail    3600    IN      A       $ip
ns1     3600    IN      A       $ip
ns2     3600    IN      A       $ip
pop     3600    IN      A       $ip
smtp    3600    IN      A       $ip
www     3600    IN      A      	$ip 

$domain.      3600    IN      MX      10 mail.$domain.



_acme-challenge 5       IN      TXT     "YRgRCOcKNeGDuBeTEcD4biN4uj_e0CDUIGEitzl519U"
$domain.      3600    IN      TXT     "v=spf1 a mx ip4:$ip ~all"

EOL
}


changenameserver() {
	cd /etc/bind/
	touch $domain.db
	echo -e "zone "$domain" { type master; file "/etc/bind/$domain.db"; };" >> named.conf

}



finish() {
	echo -e " "
	echo -e " "
	echo -e " "
	echo -e " "
	echo -e "Now Finish Nameservers :\n "
	echo -e  "ns1.$domain \t $ip  \nns2.$domain \t $ip " |lolcat -as 100
}





