Vagrant.configure("2") do |config|
  config.vm.box = "debian/bullseye64"

  config.vm.provider "virtualbox" do |v|
    v.memory = 1024 * 4
  end

  config.vm.network "private_network", ip: "192.168.56.4"

  config.vm.provision "shell" do |s|
    s.inline = ""
    Dir.glob("#{Dir.home}/.ssh/*.pub").each do |path|
      key = File.read(path).strip
      s.inline << "echo '#{key}' >> /root/.ssh/authorized_keys\n"
    end
  end
end
