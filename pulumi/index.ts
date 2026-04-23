import * as pulumi from "@pulumi/pulumi";
import * as yandex from "@pulumi/yandex";
import * as fs from "fs";

// Get configuration
const config = new pulumi.Config();
const zone = config.get("zone") || "ru-central1-d";

// Read SSH public key
const sshPublicKey = fs.readFileSync(process.env.HOME + "/.ssh/id_ed25519.pub", "utf8").trim();

// Get latest Ubuntu image
const ubuntuImage = yandex.getComputeImage({
    family: "ubuntu-2404-lts-oslogin"
});

// Create VPC network
const network = new yandex.VpcNetwork("lab4-network");

// Create subnet
const subnet = new yandex.VpcSubnet("lab4-subnet", {
    zone: zone,
    networkId: network.id,
    v4CidrBlocks: ["10.130.0.0/24"]
});

// Create security group
const securityGroup = new yandex.VpcSecurityGroup("lab4-sg", {
    name: "lab4-security-group",
    description: "Security group for Lab 4 VM",
    networkId: network.id,
    ingresses: [
        {
            protocol: "TCP",
            description: "SSH",
            v4CidrBlocks: ["0.0.0.0/0"],
            port: 22
        },
        {
            protocol: "TCP",
            description: "HTTP",
            v4CidrBlocks: ["0.0.0.0/0"],
            port: 80
        },
        {
            protocol: "TCP",
            description: "Application",
            v4CidrBlocks: ["0.0.0.0/0"],
            port: 5000
        }
    ],
    egresses: [{
        protocol: "ANY",
        description: "All outbound",
        v4CidrBlocks: ["0.0.0.0/0"],
        fromPort: 0,
        toPort: 65535
    }]
});

// Create VM instance
const vm = new yandex.ComputeInstance("lab4-vm", {
    name: "lab4-vm",
    platformId: "standard-v3",
    zone: zone,
    resources: {
        cores: 2,
        memory: 1,
        coreFraction: 20
    },
    bootDisk: {
        initializeParams: {
            imageId: ubuntuImage.then(image => image.id),
            size: 10,
            type: "network-hdd"
        }
    },
    networkInterfaces: [{
        subnetId: subnet.id,
        securityGroupIds: [securityGroup.id],
        nat: true
    }],
    metadata: {
        "ssh-keys": `ubuntu:${sshPublicKey}`
    },
    labels: {
        project: "lab4",
        environment: "learning",
        course: "devops-core"
    }
});

// Export outputs
export const vmPublicIp = vm.networkInterfaces[0].natIpAddress;
export const sshCommand = pulumi.interpolate`ssh ubuntu@${vm.networkInterfaces[0].natIpAddress}`;
