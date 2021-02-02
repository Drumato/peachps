Vagrant.configure("2") do |config|
    config.vm.provider "VirtualBox" do |v|
        v.memory = 2048
        v.customize ['modifyvm', :id, '--cableconnected1', 'on']
        v.gui = true
    end

    config.vm.define :node1 do |node|
        node.vm.box = "bento/ubuntu-20.04"
        node.vm.network :private_network, ip:"192.168.33.11"
        node.vm.hostname = "node1"
    end
    
    config.vm.define :node2 do |node|
        node.vm.box = "bento/ubuntu-20.04"
        node.vm.network :private_network, ip:"192.168.33.12"
        node.vm.hostname = "node2"
    end
end
