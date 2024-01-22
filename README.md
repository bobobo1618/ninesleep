This repository contains a program intended to replace the `dac` process on an Eight Sleep Pod 3, giving the user the ability to interact with the pod locally, without any interaction with Eight Sleep's servers.

Clearly, this project is not endorsed by Eight Sleep. You're responsible for anything you do with your pod.

To use:
- Power off your pod
- Remove the fan grille on the back of the Pod 3
- Unscrew the two screws at the top that hold on the top panel
- Pry the top plastic up, you'll reach a point where you can see clips holding the top panel to the side panels, stop here
- Remove the fabric mesh panel on the front of the device. There are two clips near the top that hook onto the plastic side panels. You can push these with a screwdriver.
- Unclip the clips near the top of the cylinder that holds the water supply and pull off the top panel. You should now have access to the logic board.
- There's a little daughterboard that looks like it's plugged into a RAM slot. Remove the screws from this board, and the antenna.
- Gently pull the board up, away from the thermal putty.
- Pull the board out.
- Remove the glue from the MicroSD slot that has been revealed
- Remove the MicroSD card from the slot
- Modify the rootfs.tar.gz file located at `/opt/images/Yocto/rootfs.tar.gz` on the first partition of the MicroSD card.
    - Set the root password in `/etc/shadow`
    - Add a `NetworkManager` configuration file at `/etc/NetworkManager/system-connections/<something>.nmconnection`
    - Add your SSH key to `/etc/ssh/authorized_keys` (optionally, remove Eight Sleep's key while you're there)
- Insert the MicroSD card
- Insert the logic daughterboard
- Reattach the antenna
- Hold the smaller button on the back of the pod, next to the power cable. While the button is held in, plug the power in.
- After a few seconds, you should see the pod's light flash green. This indicates the device is performing a factory reset, using the `rootfs.tar.gz` file.
- Eventually it'll stop flashing green and connect to wifi. You should now be able to ssh into your device using the account `rewt`. You can also log in as `root` by logging in as `rewt`, typing `su` and entering the root password.

You can now compile this program for the Pod 3: `cargo build --target aarch64-unknown-linux-musl` (musl is used so that a static binary will work). Copy it to your Pod over ssh and you should be able to run it, although you'll need to run `systemctl stop dac` as root first to shut down the stock `dac`, which listens on the relevant unix socket.

You may want to disable Eight Sleep's updates and telemetry. You can do that with: `systemctl disable --now swupdate-progress swupdate defibrillator eight-kernel telegraf vector`. The `frankenfirmware` binary will still send data to `raw-api-upload.8slp.net`. If you want to deal with that, add `raw-api-upload.8slp.net` to your `/etc/hosts` file.