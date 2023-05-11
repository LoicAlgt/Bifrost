#!/bin/bash

# Vérifier si MySQL est installé
if ! command -v mysql &> /dev/null
then
    echo "MySQL n'est pas installé, en cours d'installation ..."
    sudo apt install mysql-server
fi

# Connexion à MySQL et exécution des requêtes
mysql -u root -p -e "
    CREATE DATABASE IF NOT EXISTS crypto;
    USE crypto;
    CREATE TABLE IF NOT EXISTS utilisateurs (nom VARCHAR(50), prenom VARCHAR(50), code VARCHAR(50),adresse VARCHAR(50),pays VARCHAR(50),ville VARCHAR(50),organisation VARCHAR(50));
    CREATE USER 'crypto'@'localhost' IDENTIFIED BY 'crypto';
    GRANT ALL PRIVILEGES ON crypto.* TO 'crypto'@'localhost';
"

