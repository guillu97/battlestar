# Constantes Partagées

## 📋 Source de Vérité

Le fichier `game-constants.toml` à la racine du projet est **la seule source de vérité** pour toutes les constantes physiques du jeu.

```toml
[physics]
thrust_accel = 1000.0     # pixels/sec²
rotation_speed = 3.0      # radians/sec
max_speed = 2000.0        # pixels/sec
drag = 0.98               # velocity multiplier per frame
world_limit = 800.0       # world boundary for wrapping

[gameplay]
ship_radius = 25.0        # pixels
```

## 🔧 Comment ça fonctionne

1. **Éditer les constantes** : Modifiez uniquement `game-constants.toml`
2. **Build automatique** : Les scripts `build.rs` génèrent automatiquement les fichiers constants à partir du TOML
3. **Synchronisation garantie** : Client et serveur utilisent exactement les mêmes valeurs

## 🚀 Déploiement

### Serveur (Fly.io)
```bash
cd server
cargo build --release
fly deploy
```

### Client (Vercel)
```bash
cd client
trunk build --release
# Vercel déploie automatiquement depuis le repo
```

## ⚠️ Important

- **NE PAS** modifier `client/src/constants.rs` ou `server/src/constants.rs` manuellement
- Ces fichiers incluent du code auto-généré via `include!(concat!(env!("OUT_DIR"), "/generated_constants.rs"))`
- Toute modification sera **écrasée** au prochain build

## 📝 Ajouter une nouvelle constante

1. Ajoutez-la dans `game-constants.toml` sous la section appropriée
2. Modifiez les `build.rs` pour extraire et générer la nouvelle constante
3. Rebuild les deux projets

## ✅ Avantages

- ✓ Une seule source de vérité
- ✓ Impossible de désynchroniser client/serveur
- ✓ Fonctionne avec des déploiements séparés (Fly.io + Vercel)
- ✓ Validation au moment de la compilation
- ✓ Pas de duplication de code
