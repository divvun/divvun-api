# Deployment

There's a small Ansible Playbook to help with deployment.

## Requirements

You need to [install Ansible](https://docs.ansible.com/ansible/latest/installation_guide/intro_installation.html) on the operator computer (the computer which will be executing the deployment) and make sure you have created a `.env` file from the `.env_sample`. You can either keep it in the same folder as the Ansible playbook or specify the location with the variable `env_file_path`. 

On the target server you need to have made sure the following things exist:

- Created a regular user with sudo priviliges (default: *ubuntu*)
- Created an API user with which the API will run (default: *api*)
- Have SSH access
- The target machine has python installed (e.g. `apt-get install python`)

## Running

In the same folder as the `playbook.yml` file you can run the following commands to deploy:

- Specifying the host only (*the trailing comma is important*)
    ```sh
    ansible-playbook -i 111.111.111.111, playbook.yml
    ```
- If the sudo user is not configured to run sudo without a password:
    ```sh
    ansible-playbook -i 111.111.111.111, -K playbook.yml
    ```
- If you wanna change the default usernames or domain name you can do it either in the `playbook.yml` or by passing the respective variables (sudo_user, api_server_user, domain):
    ```sh
    ansible-playbook -i 111.111.111.111, --extra-vars "sudo_user=admin api_server_user=web" playbook.yml
    ```

## NOTES

This has been tested on Ubuntu 18.04.

This does not load the necessary grammar and spell checker data files. It just creates the needed directories but has no way for now to put the expected files per language there.

Set the `admin_email` variable to receive emails from let's encrypt when it's time to renew the HTTPS certificate and such.