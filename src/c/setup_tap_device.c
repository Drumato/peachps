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

// Rustの定義と同じ構造のTapDeviceを定義
typedef struct TapDevice
{
    int32_t fd;
    uint8_t mac_addr[6];
} TapDevice;

const char *if_name_g = "tap0";

// TAPデバイスの作成
int create_tap_device(char *device_path, TapDevice *tap_device)
{
    // TAPデバイスのfdをもらう
    int fd;
    struct ifreq ifr;
    memset(&ifr, 0, sizeof(ifr));
    if ((fd = open(device_path, O_RDWR)) < 0)
    {
        return -1;
    }
    tap_device->fd = fd;

    // IFF_TAP等を渡して実際にデバイスを作成する
    strncpy(ifr.ifr_name, if_name_g, strlen(if_name_g));
    ifr.ifr_ifru.ifru_flags = IFF_TAP | IFF_NO_PI;

    if (ioctl(fd, TUNSETIFF, (void *)&ifr) < 0)
    {
        close(fd);
        return -1;
    }

    return 0;
}

// デバイスにMAC addressを割り当てる
int set_mac_addr_to_dev(TapDevice *tap_device)
{
    int fd;
    struct ifreq ifr;
    memset(&ifr, 0, sizeof(ifr));

    // MAC addressを得るためにソケットを開く
    // このソケットを通してアドレスの受け渡しを行う
    fd = socket(AF_INET, SOCK_DGRAM, 0);
    if (fd == -1)
    {
        perror("socket");
        return -1;
    }

    ifr.ifr_addr.sa_family = AF_INET;
    strncpy(ifr.ifr_name, if_name_g, sizeof(ifr.ifr_name) - 1);

    // 実際にMAC addressを要求する
    if (ioctl(fd, SIOCGIFHWADDR, &ifr) == -1)
    {
        close(fd);
        return -1;
    }
    memcpy(tap_device->mac_addr, ifr.ifr_hwaddr.sa_data, sizeof(uint8_t) * 6);
    close(fd);

    return 0;
}

// IPアドレスをTAPデバイスに割り当てる．
// また，TAPデバイスの起動も行う．
int allocate_ip_addr_to_dev()
{
    if (system("ip a add 10.0.0.2/24 dev tap0") != 0)
    {
        return -1;
    }
    if (system("ip link set tap0 up") != 0)
    {
        return -1;
    }

    return 0;
}

int _setup_tap_dev(char *device_path, TapDevice *tap_device)
{
    memset(tap_device, 0, sizeof(TapDevice));

    if (create_tap_device(device_path, tap_device) != 0)
    {
        return 0;
    }

    if (set_mac_addr_to_dev(tap_device) != 0)
    {
        return -1;
    }

    if (allocate_ip_addr_to_dev() != 0)
    {
        return -1;
    }

    return 0;
}