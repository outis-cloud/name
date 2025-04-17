docker run -itd   -p 53:53/udp   -p 53:53/tcp   -v $(pwd)/zerooo.ir.zone:/var/lib/powerdns/zones/zerooo.ir.zone   -v $(pwd)/named.conf:/etc/powerdns/named.conf   -v $(pwd)/pdns.conf:/etc/powerdns/pdns.d/pdns.conf   outis92/pdns


