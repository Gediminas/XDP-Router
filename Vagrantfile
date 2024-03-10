#!/usr/bin/env ruby

BOX         = "debian/bookworm64"
BOX_VERSION = "12.20240212.1"
SERVER_IP   = "192.168.171.10"
CLIENT_IP   = "192.168.171.200"

Vagrant.configure("2") do |config|
  config.vm.box = BOX
  config.vm.box_version = BOX_VERSION
  config.vm.box_check_update = false

  config.vm.define "server" do |server|
    server.vm.hostname = "xdp-router-server"
    server.vm.network "private_network", ip: SERVER_IP
    server.vm.provision :shell, privileged: true,  :path => "./asset/setup_server.sh"
    server.vm.provision :shell, privileged: false, :path => "./asset/install_rustup.sh"
    server.vm.provision :shell, privileged: false, :path => "./asset/prepare_rustup.sh"

    server.vm.provider :virtualbox do |vbox|
      vbox.cpus   = 4
      vbox.memory = 2048
      vbox.customize ["modifyvm", :id, "--ioapic", "on"] # Enable all cores
    end

    server.vm.post_up_message = "vagrant ssh server (#{SERVER_IP})"
  end

  config.vm.define "client" do |client|
    client.vm.hostname = "xdp-router-client"
    client.vm.network "private_network", ip: CLIENT_IP
    client.vm.provision :shell, :path => "./asset/setup_client.sh"
    client.vm.post_up_message = "vagrant ssh client (#{CLIENT_IP})"
  end

end
