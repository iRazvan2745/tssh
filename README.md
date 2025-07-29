# TSSH - Tailscale SSH

TSSH is a simple CLI application designed to streamline your SSH experience with Tailscale.

## Screenshots

| ![TSSH Screenshot 1](https://irazz.b-cdn.net/tssh/tssh1.png) | ![TSSH Screenshot 2](https://irazz.b-cdn.net/tssh/tssh2.png) |
|:------------------------------------------------------------:|:------------------------------------------------------------:|
|                         _Categories_                         |                      _Allowed category_                      |



## Config

The config is found at /home/$USER/.config/tssh/config.json

Example: 
```json
{
  "user": "root",
  "allowed": {
    "personal": [
      "ams-1","ams-2","ams-3","ro-1","ro-storage-1"
    ],
    "other-category": [
      "biz-1","biz-2"
    ]
   }
}
```