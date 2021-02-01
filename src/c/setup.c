
#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <linux/if.h>
#include <linux/if_ether.h>
#include <linux/if_packet.h>
#include <linux/if_tun.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <unistd.h>

struct NetDevice
{
    int32_t fd;
    uint8_t mac_addr[6];
};
char *if_name_g;

typedef struct NetDevice RawSocket;
// typedef struct NetDevice TapDevice;

// static int create_tap_device(char *device_path, TapDevice *tap_device);
static int set_mac_address(char *interface_name, uint8_t *mac_addr, struct ifreq *ifr);
static int try_open_raw_socket();
static int find_dev_interface_index(struct ifreq *ifr, int dev_fd);
static int bind_address_to_socket(int dev_fd, struct sockaddr_ll *sock_addr, struct ifreq *ifr);
static int set_promiscuous_mode(int fd, struct ifreq *ifr);
// static int allocate_ip_addr_to_dev();

/*
int _setup_tap_dev(char *device_path, TapDevice *tap_device)
{
    struct ifreq ifr;
    if_name_g = "tap0";
    memset(tap_device, 0, sizeof(TapDevice));

    if (create_tap_device(device_path, tap_device) != 0)
    {
        return 0;
    }

    if (set_mac_address("tap0", tap_device->mac_addr, &ifr) != 0)
    {
        return -1;
    }

    if (allocate_ip_addr_to_dev() != 0)
    {
        return -1;
    }

    return 0;
}
*/

int _setup_raw_sock(char *interface_name, RawSocket *raw_sock)
{
    struct ifreq ifr;
    struct sockaddr_ll sock_addr;
    memset(&ifr, 0, sizeof(ifr));
    memset(&sock_addr, 0, sizeof(sock_addr));

    if ((raw_sock->fd = try_open_raw_socket()) == -1)
    {
        perror("failed to open raw socket");
        return -1;
    }

    strncpy(ifr.ifr_name, interface_name, sizeof(ifr.ifr_name) - 1);
    if (find_dev_interface_index(&ifr, raw_sock->fd) == -1)
    {
        perror("failed to find network device interface index");
        return -1;
    }

    if (bind_address_to_socket(raw_sock->fd, &sock_addr, &ifr) == -1)
    {
        perror("failed to bind address to raw socket");
        return -1;
    }

    if (set_promiscuous_mode(raw_sock->fd, &ifr) == -1)
    {
        perror("failed to set promiscuous mode");
        return -1;
    }

    // get physical device's mac address
    memset(&ifr, 0, sizeof(ifr));
    if (set_mac_address(interface_name, raw_sock->mac_addr, &ifr) == -1)
    {
        return -1;
    }

    return 0;
}

static int try_open_raw_socket()
{
    int fd = socket(AF_PACKET, SOCK_RAW, htons(ETH_P_ALL));
    return fd;
}

static int find_dev_interface_index(struct ifreq *ifr, int dev_fd)
{
    if (ioctl(dev_fd, SIOCGIFINDEX, ifr) == -1)
    {
        return -1;
    }
    return 0;
}

// ソケットにアドレスを割り当てる
static int bind_address_to_socket(int dev_fd, struct sockaddr_ll *sock_addr, struct ifreq *ifr)
{
    sock_addr->sll_family = PF_PACKET;
    sock_addr->sll_protocol = htons(ETH_P_ALL);
    sock_addr->sll_ifindex = ifr->ifr_ifindex;

    if (bind(dev_fd, (struct sockaddr *)sock_addr, sizeof(*sock_addr)) == -1)
    {
        return -1;
    }

    return 0;
}

static int set_promiscuous_mode(int fd, struct ifreq *ifr)
{
    // set IFF_PROMISC flag
    if (ioctl(fd, SIOCGIFFLAGS, ifr) == -1)
    {
        return -1;
    }
    ifr->ifr_flags = ifr->ifr_flags | IFF_PROMISC | IFF_UP | IFF_RUNNING;

    if (ioctl(fd, SIOCSIFFLAGS, ifr) == -1)
    {
        return -1;
    }
    return 0;
}

static int set_mac_address(char *interface_name, uint8_t *mac_addr, struct ifreq *ifr)
{
    int fd = socket(AF_INET, SOCK_DGRAM, 0);
    if (fd == -1)
    {

        return -1;
    }

    ifr->ifr_addr.sa_family = AF_INET;
    strncpy(ifr->ifr_name, interface_name, sizeof(ifr->ifr_name) - 1);
    if (ioctl(fd, SIOCGIFHWADDR, ifr) == -1)
    {
        close(fd);
        return -1;
    }

    memcpy(mac_addr, ifr->ifr_hwaddr.sa_data, 6);
    close(fd);

    return 0;
}

// TAPデバイスの作成
/*
static int create_tap_device(char *device_path, TapDevice *tap_device)
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
*/

// IPアドレスをTAPデバイスに割り当てる．
// また，TAPデバイスの起動も行う．
/*
static int allocate_ip_addr_to_dev()
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
*/