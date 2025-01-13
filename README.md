# NameShift: DNS Management Script

NameShift is a DNS management script designed for the installation, configuration, and management of CoreDNS. This script provides various features such as adding domains, creating DNS records, viewing logs, and backing up CoreDNS settings.

## Features

- Add a new domain: Add a new domain to the system
- Add a DNS record: Add a new DNS record (A, CNAME, AAAA, etc.) for a domain.
- Remove a DNS record: Remove a DNS record from a domain.
- List all domains: List all the domains that have been configured
- Reload CoreDNS: Reload the CoreDNS configuration.
- Backup zones: Create a backup of DNS zones.
- Restore zones: Restore DNS settings from a previous backup.
- View logs: View system logs.

# Use the script

````
```
curl -sLO https://raw.githubusercontent.com/Niihil/NameShift/main/run && sudo bash run
```
