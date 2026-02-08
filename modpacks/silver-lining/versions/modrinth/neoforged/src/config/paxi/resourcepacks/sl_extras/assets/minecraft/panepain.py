COLORS = [
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black"
]

state_template = ""
noside_alt_template = ""
noside_template = ""
post_template = ""
side_alt_template = ""
side_template = ""

with open("blockstates/glass_pane.json", "rt") as f: state_template = f.read()
with open("models/block/glass_pane_noside_alt.json", "rt") as f: noside_alt_template = f.read()
with open("models/block/glass_pane_noside.json", "rt") as f: noside_template = f.read()
with open("models/block/glass_pane_post.json", "rt") as f: post_template = f.read()
with open("models/block/glass_pane_side_alt.json", "rt") as f: side_alt_template = f.read()
with open("models/block/glass_pane_side.json", "rt") as f: side_template = f.read()

for color in COLORS:
    pane_name = f"{color}_stained_glass_pane"
    block_name = f"{color}_stained_glass"
    with open(f"blockstates/{pane_name}.json", "wt") as f:
        f.write(state_template.replace("glass_pane", pane_name))
    with open(f"models/block/{pane_name}_noside_alt.json", "wt") as f:
        f.write(noside_alt_template.replace("block/glass_pane", "block/" + pane_name))
    with open(f"models/block/{pane_name}_noside.json", "wt") as f:
        f.write(noside_template.replace("block/glass_pane", "block/" + pane_name))
    with open(f"models/block/{pane_name}_post.json", "wt") as f:
        f.write(post_template.replace("block/glass_pane", "block/" + pane_name)\
            .replace("block/glass\"", "block/" + block_name + "\""))
    with open(f"models/block/{pane_name}_side_alt.json", "wt") as f:
        f.write(side_alt_template.replace("block/glass_pane", "block/" + pane_name)\
            .replace("block/glass\"", "block/" + block_name + "\""))
    with open(f"models/block/{pane_name}_side.json", "wt") as f:
        f.write(side_template.replace("block/glass_pane", "block/" + pane_name)\
            .replace("block/glass\"", "block/" + block_name + "\""))