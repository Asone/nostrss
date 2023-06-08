# Nostrss-cli

Nostrss-cli is a CLI program that provides helpers to manage live instance of Nostrss.

Default behavior of the CLI will only update the running instance configuration without modifying the loaded configuration files. 

If you want to persist the configuration modifications you can use the `--save` flag as first argument to update the configuration files. 

e.g: 
> nostrss-cli --save profile delete reddit

The `--save` flag can be used to update profiles and feeds config files and works when instructing 
`add` or `delete` command. 

Note that, when using the flag, the config file will be overwritten with the full configuration of the instance. For example, if you add a profile without the flag and then another one with the `--save` flag, both new profiles will be written in the configuration file. 

## State commands

| Command | Description |
|-|-|
| nostrss-cli state | Ensures the core can be reached | 


## Profiles

| Command | Description | 
|-|-|
| nostrss-cli profile list | Lists the profiles |
| nostrss-cli profile add | Add a new profile | 
| nostrss-cli profile delete | Remove a profile. Beware, you can not delete default profile for stability issues | 
| nostrss-cli profile info | Get info of a specific profile | 

### Feeds

| Command | Description | 
|-|-|
| nostrss-cli feed list | Lists the feeds |
| nostrss-cli feed add | Add a new feed  | 
| nostrss-cli feed delete | Remove a feed | 
| nostrss-cli feed info | Get info of a specific feed | 
