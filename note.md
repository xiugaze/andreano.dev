
`flake.nix`:
```nix
{
    # ...
    inputs = {
        andreano-dev.url = "url";
    }

    outputs = inputs@{ nixpkgs, andreano-dev} :
        let 
            hostname = "YOUR HOSTNAME";
        in {
            nixosConfigurations."${hostname}" = nixpkgs.lib.nixosSystem {
                system = "x86_64-linux";
                specialArgs = { inherit nixpkgs-unstable andreano-dev; };
                modules = [
                    ./configuration.nix
                ];
            }
        }
    }
}
```

`configuration.nix`
```nix
{ config, pkgs, ...}@inputs:

{
    # ...
    environment.systemPackages = with pkgs; [
        inputs.andreano-dev.${pkgs.system}.default;
    ];
    #...
}


```

## SOPS notes
```

read -S SSH_TO_AGE_PASSPHRASE; export SSH_TO_AGE_PASSPHRASE
ssh-to-age -private-key -i ~/.ssh/id_ed25519 > ~/.config/sops/age/keys.txt
```


