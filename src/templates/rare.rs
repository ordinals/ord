import re

# Ouvre ton fichier source Rust
with open('src/ton_fichier.rs', 'r') as f:
    code = f.read()

# Extrait les IDs qui correspondent à tes inscriptions (adapte le regex selon ton code)
# Exemple : si tes IDs sont stockés dans des constantes ou des vecteurs
ids = re.findall(r'ID_INSCRIPTION\s*=\s*"([a-f0-9]+)"', code)

with open('inscriptions_rares.txt', 'w') as out:
    for id in ids:
        out.write(f"{id}\n")

print(f"Extraction terminée : {len(ids)} IDs trouvés.")
