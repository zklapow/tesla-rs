To use the Tesla command-line utility, you need to 
define a config file (default name is ".teslac", default location 
is the home folder) and it needs to contain a couple of configs.

Exemple config file :

```
[global]
api_token = "____SOME_API_TOKEN_OBTAINED_FROM_A_LOGIN_ENDPOINT____"
logspec = "info"
default_vehicle = "CAR_NAME_GOES_HERE"
```

If the `default_vehicle` config is omitted, it will need to be 
passed on the command-line.

Ex :
```
teslac --vehicle "Red Stapler" get_all_data
```
