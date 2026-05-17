output "vm_public_ip" {
  description = "Public IP address of the VM"
  value       = yandex_compute_instance.lab_vm.network_interface[0].nat_ip_address
}

output "vm_private_ip" {
  description = "Private IP address of the VM"
  value       = yandex_compute_instance.lab_vm.network_interface[0].ip_address
}

output "ssh_connection_command" {
  description = "Command to connect via SSH"
  value       = "ssh ubuntu@${yandex_compute_instance.lab_vm.network_interface[0].nat_ip_address}"
}

output "vm_id" {
  description = "VM instance ID"
  value       = yandex_compute_instance.lab_vm.id
}
