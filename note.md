
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

