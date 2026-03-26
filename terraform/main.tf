terraform {
  required_providers {
    yandex = {
      source = "yandex-cloud/yandex"
    }
  }
  required_version = ">= 0.13"
}

provider "yandex" {
  service_account_key_file = "terraform-key.json"
  cloud_id                 = var.yc_cloud_id
  folder_id                = var.yc_folder_id
  zone                     = var.zone
}

# Get the latest Ubuntu 24.04 image
data "yandex_compute_image" "ubuntu" {
  family = "ubuntu-2404-lts-oslogin"
}

# Create VPC network
resource "yandex_vpc_network" "lab_network" {
  name = "lab4-network"
}

# Create subnet
resource "yandex_vpc_subnet" "lab_subnet" {
  name           = "lab4-subnet"
  zone           = var.zone
  network_id     = yandex_vpc_network.lab_network.id
  v4_cidr_blocks = ["10.130.0.0/24"]
}

# Create security group
resource "yandex_vpc_security_group" "lab_sg" {
  name        = "lab4-security-group"
  description = "Security group for Lab 4 VM"
  network_id  = yandex_vpc_network.lab_network.id

  # SSH access
  ingress {
    protocol       = "TCP"
    description    = "SSH"
    v4_cidr_blocks = ["0.0.0.0/0"]
    port           = 22
  }

  # HTTP access
  ingress {
    protocol       = "TCP"
    description    = "HTTP"
    v4_cidr_blocks = ["0.0.0.0/0"]
    port           = 80
  }

  # App port
  ingress {
    protocol       = "TCP"
    description    = "Application"
    v4_cidr_blocks = ["0.0.0.0/0"]
    port           = 8000
  }

  # App port
  ingress {
    protocol       = "TCP"
    description    = "Application"
    v4_cidr_blocks = ["0.0.0.0/0"]
    port           = 8001
  }

  # Allow all outbound traffic
  egress {
    protocol       = "ANY"
    description    = "All outbound"
    v4_cidr_blocks = ["0.0.0.0/0"]
    from_port      = 0
    to_port        = 65535
  }
}

# Create VM instance
resource "yandex_compute_instance" "lab_vm" {
  name        = "lab4-vm"
  platform_id = "standard-v3"
  zone        = var.zone

  resources {
    cores         = 2
    memory        = 1
    core_fraction = 20 # Free tier: 20% CPU guarantee
  }

  boot_disk {
    initialize_params {
      image_id = data.yandex_compute_image.ubuntu.id
      size     = 10
      type     = "network-hdd"
    }
  }

  network_interface {
    subnet_id          = yandex_vpc_subnet.lab_subnet.id
    security_group_ids = [yandex_vpc_security_group.lab_sg.id]
    nat                = true # Assign public IP
  }

  metadata = {
    ssh-keys = "ubuntu:${file(var.ssh_public_key_path)}"
  }

  labels = {
    project     = "lab4"
    environment = "learning"
    course      = "devops-core"
  }
}
