#include <errno.h>
#include <fcntl.h>
#include <linux/if.h>
#include <linux/if_tun.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <unistd.h>

typedef struct TapDevice
{
    int32_t fd;
    uint8_t mac_addr[6];
} TapDevice;

int setup(char *device_path, char *interface_name, TapDevice *tap_device)
{
    int fd, err;
    struct ifreq ifr;
    if ((fd = open(device_path, O_RDWR)) < 0)
    {
        return -1;
    }
    tap_device->fd = fd;
    memset(&ifr, 0, sizeof(ifr));
    strncpy(ifr.ifr_name, interface_name, IFNAMSIZ);
    ifr.ifr_ifru.ifru_flags = IFF_TAP | IFF_NO_PI;

    if ((err = ioctl(fd, TUNSETIFF, (void *)&ifr)) < 0)
    {
        fprintf(stderr, "Failed to creat tap device\n");
        perror("ioctl");
        close(fd);
        return -1;
    }

    memset(&ifr, 0, sizeof(ifr));
    int soc;

    soc = socket(AF_INET, SOCK_DGRAM, 0);
    if (soc == -1)
    {
        perror("socket");
        return -1;
    }
    ifr.ifr_addr.sa_family = AF_INET;
    strncpy(ifr.ifr_name, interface_name, sizeof(ifr.ifr_name) - 1);
    if (ioctl(soc, SIOCGIFHWADDR, &ifr) == -1)
    {
        perror("ioctl [SIOCGIFHWADDR]");
        close(soc);
        return -1;
    }
    memcpy(tap_device->mac_addr, ifr.ifr_hwaddr.sa_data, 6);
    close(soc);
    return 0;
}