#!/usr/bin/env bash
# Script pour croiser tes inscriptions avec la liste des raretés

# Télécharge la liste de rareté si tu ne l'as pas déjà
curl -s https://ordinals.com/rare.txt > rare_source.txt

# Ton fichier d'inscriptions (à remplacer par ton fichier de travail)
INPUT_INSCRIPTIONS="mes_inscriptions.txt"

echo "Croisement des données en cours..."

# Utilise 'grep' pour extraire uniquement les ID qui apparaissent dans rare.txt
grep -Ff rare_source.txt "$INPUT_INSCRIPTIONS" > inscriptions_rares_reunies.txt

echo "Terminé. Les éléments rares ont été réunis dans : inscriptions_rares_reunies.txt"
