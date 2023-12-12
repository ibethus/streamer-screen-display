# Streamer screen display
Ce petit programme a pour but de récupérer les informations de lecture du deamon MPD et de les afficher sur un écran externe.

## Waveshare e-screen 2in9
Le module utilisé pour l'affichage est [un écran e-ink Waveshare de 2"9](https://www.waveshare.com/wiki/2.9inch_e-Paper_Module).
Il doit être connecté en SPI avec le cable fourni en suivant le layout suivant sur le pins de raspberry pi : 

|e-Paper	| Raspberry Pi |
|---------|--------------|
|VCC|	3.3V|
|GND|	GND|
|DIN|	19|
|CLK|	23|
|CS|	24|
|DC|	22|
|RST|	11|
|BUSY| 18|

![gpio layout](https://pi4j.com/1.2/images/j8header-3b.png)
