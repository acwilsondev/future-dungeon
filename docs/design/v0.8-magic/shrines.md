# Shrines

Shrines appear as `&` colored as their origin.

Players may meditate at *Shrines* to attempt to raise their Mana Pool by one Mana.

1. Has this shrine been attempted before? If so, fail.
2. Does this character have five mana already? If so, fail.

Otherwise, they make a CHA check equal to

`10 + (Total Maximum Mana of All Colors)`

On failure, they receive a message `The shrine is silent. Peace be with you.` The shrine is marked as attempted and cannot be tried again.

On success, they receive a message `The shrine resonates with mystic energy. Raise your [Color] mana by one (1) point?`

The color depends on the origin of the Shrine. If the player confirms, the mana is granted and the shrine is marked as attempted. If the player declines the prompt, the shrine is **not** marked as attempted and may be interacted with again later.
