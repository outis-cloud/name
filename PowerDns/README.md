# PowerDNS Stack

This directory contains the PowerDNS stack for **NameShift**, including three services:

1. **PowerDNS**: The authoritative DNS server.
2. **Database**: MySQL database for storing DNS records.
3. **Admin Panel**: Web interface for managing domains (PowerDNS-Admin).

## Prerequisites

- Docker & Docker Compose installed.
- A `.env` file in this folder with:
```
MYSQL_ROOT_PASSWORD= PASSWORD
MYSQL_DATABASE=pdns_admin_guilt
MYSQL_USER=pdns_admin
MYSQL_PASSWORD=PASSWORD
PDNS_DB_PASSWORD=PASSWORD
PDNS_API_KEY= API_KEY
PDNS_ADMIN_USER=admin_panel
PDNS_ADMIN_PASSWORD=admin_panel
SECRET_KEY= SECRET_KEY=
```

## Usage

```
git clone https://github.com/Guilt92/NameShift.git
cd NameShift/PowerDns
./pdns
```

Access services:
- DNS queries: port **53**
- PowerDNS API: port **8088**
- Admin Panel: http://ip-server:9191



