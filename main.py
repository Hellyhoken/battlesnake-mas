
# Welcome to
# __________         __    __  .__                               __
# \______   \_____ _/  |__/  |_|  |   ____   ______ ____ _____  |  | __ ____
#  |    |  _/\__  \\   __\   __\  | _/ __ \ /  ___//    \\__  \ |  |/ // __ \
#  |    |   \ / __ \|  |  |  | |  |_\  ___/ \___ \|   |  \/ __ \|    <\  ___/
#  |________/(______/__|  |__| |____/\_____>______>___|__(______/__|__\\_____>
#
# This file can be a nice home for your Battlesnake logic and helper functions.
#
# To get you started we've included code to prevent your Battlesnake from moving backwards.
# For more info see docs.battlesnake.com

import random
import typing

color = "#ee1111"

# info is called when you create your Battlesnake on play.battlesnake.com
# and controls your Battlesnake's appearance
# TIP: If you open your Battlesnake URL in a browser you should see this data
def info() -> typing.Dict:
    print("INFO")

    return {
        "apiversion": "1",
        "author": "Group 18",  # TODO: Your Battlesnake Username
        "color": "#11ee11",  # TODO: Choose color
        "head": "default",  # TODO: Choose head
        "tail": "default",  # TODO: Choose tail
    }


# start is called when your Battlesnake begins a game
def start(game_state: typing.Dict):
    print("GAME START")


# end is called when your Battlesnake finishes a game
def end(game_state: typing.Dict):
    print("GAME OVER\n")


# move is called on every turn and returns your next move
# Valid moves are "up", "down", "left", or "right"
# See https://docs.battlesnake.com/api/example-move for available data
def move(game_state: typing.Dict) -> typing.Dict:

    is_move_safe = {"up": True, "down": True, "left": True, "right": True}

    # We've included code to prevent your Battlesnake from moving backwards
    my_head = game_state["you"]["body"][0]  # Coordinates of your head
    my_neck = game_state["you"]["body"][1]  # Coordinates of your "neck"

    if my_neck["x"] < my_head["x"]:  # Neck is left of head, don't move left
        is_move_safe["left"] = False

    elif my_neck["x"] > my_head["x"]:  # Neck is right of head, don't move right
        is_move_safe["right"] = False

    elif my_neck["y"] < my_head["y"]:  # Neck is below head, don't move down
        is_move_safe["down"] = False

    elif my_neck["y"] > my_head["y"]:  # Neck is above head, don't move up
        is_move_safe["up"] = False

    # TODO: Step 1 - Prevent your Battlesnake from moving out of bounds
    board_width = game_state['board']['width']
    board_height = game_state['board']['height']
    if my_head['x'] == 0:
        is_move_safe['left'] = False
    elif my_head['x'] == board_width - 1:
        is_move_safe['right'] = False
    if my_head['y'] == 0:
        is_move_safe['down'] = False
    elif my_head['y'] == board_height - 1:
        is_move_safe['up'] = False

    # TODO: Step 2 - Prevent your Battlesnake from colliding with itself
    my_body = game_state['you']['body']
    for i in range(3, len(my_body)):
        if my_head['x'] == my_body[i]['x'] and my_head['y'] == my_body[i]['y']+1:
            is_move_safe['down'] = False
        elif my_head['x'] == my_body[i]['x'] and my_head['y'] == my_body[i]['y']-1:
            is_move_safe['up'] = False
        elif my_head['x'] == my_body[i]['x']+1 and my_head['y'] == my_body[i]['y']:
            is_move_safe['left'] = False
        elif my_head['x'] == my_body[i]['x']-1 and my_head['y'] == my_body[i]['y']:
            is_move_safe['right'] = False   

    # TODO: Step 3 - Prevent your Battlesnake from colliding with other Battlesnakes
    # opponents = game_state['board']['snakes']

    # Are there any safe moves left?
    safe_moves = []
    for move, isSafe in is_move_safe.items():
        if isSafe:
            safe_moves.append(move)



    if len(safe_moves) == 0:
        print(f"MOVE {game_state['turn']}: No safe moves detected! Moving down")
        return {"move": "down"}

    
    stored_lengths = {}
    move_lengths = []
    for move in safe_moves:
        passed = {}
        new_pos = get_new_pos(my_head, move)
        max_len = get_max_length(new_pos, game_state['board'], get_safe_moves(move), len(my_body)-1, stored_lengths, passed)

        #print(f"Max length for {move} is {max_len} (current length is {len(my_body)})")

        move_lengths.append(max_len)
    
    # If we have no move that is longer than our current length, we should take the longest one
    if max(move_lengths) < len(my_body)-1:
        print("No move is longer than current length, taking the longest one")
        max_index = move_lengths.index(max(move_lengths))
        next_move = safe_moves[max_index]
    else:
        # Choose a random move from the safe ones
        long_moves = []
        for i in range(len(move_lengths)):
            if move_lengths[i] >= len(my_body)-1:
                long_moves.append(safe_moves[i])
        next_move = random.choice(long_moves)

    # TODO: Step 4 - Move towards food instead of random, to regain health and survive longer
    # food = game_state['board']['food']

    print(f"MOVE {game_state['turn']}: {next_move}")
    return {"move": next_move}

def detect_collision(pos, board):
    if pos['x'] < 0 or pos['x'] >= board['width'] or pos['y'] < 0 or pos['y'] >= board['height']:
        return True

    for hazard in board['hazards']:
        if pos['x'] == hazard['x'] and pos['y'] == hazard['y']:
            return True
        
    for snake in board['snakes']:
        for body in snake['body']:
            if pos['x'] == body['x'] and pos['y'] == body['y']:
                return True
    
    return False

def get_max_length(pos, board, safe_moves, max_length, stored_lengths = {}, passed = {}):
    h_pos = hash_pos(pos)
    if h_pos in stored_lengths:
        maximum = 0
        for move in safe_moves:
            if stored_lengths[h_pos][move_to_num[move]] == 0:
                stored_lengths[h_pos][move_to_num[move]] = get_max_length(get_new_pos(pos, move), board, get_safe_moves(move), max_length - 1, stored_lengths, passed) + 1

            if stored_lengths[h_pos][move_to_num[move]] > maximum:
                maximum = stored_lengths[h_pos][move_to_num[move]]
        #print(f"Max length for {h_pos} is {maximum}")
        return maximum
    
    if h_pos in passed:
        #print(f"Already passed {h_pos}")
        return 0
    
    if detect_collision(pos, board):
        #print(f"Collision detected at {h_pos}")
        return 0
    
    if max_length == 0:
        #print(f"Max length is 0 at {h_pos}")
        return 0
    
    passed[h_pos] = True
    lengths = [0, 0, 0, 0]

    for move in safe_moves:
        new_pos = get_new_pos(pos, move)
        
        lengths[move_to_num[move]] = get_max_length(new_pos, board, get_safe_moves(move), max_length - 1, stored_lengths, passed) + 1

    #print(f"Max length for {h_pos} is {lengths}")
    stored_lengths[h_pos] = lengths
    return max(lengths)

def hash_pos(pos):
    return str(pos['x']) + "," + str(pos['y'])


def get_new_pos(pos, move):
    new_pos = pos.copy()
    if move == "up":
        new_pos['y'] += 1
    elif move == "down":
        new_pos['y'] -= 1
    elif move == "left":
        new_pos['x'] -= 1
    elif move == "right":
        new_pos['x'] += 1
    return new_pos

def get_safe_moves(old_move):
    safe_moves = []
    for move, num in move_to_num.items():
        if move == old_move or move_to_num[old_move]%2 != num%2:
            safe_moves.append(move)
    return safe_moves

move_to_num = {
    "up": 0,
    "left": 1,
    "down": 2,
    "right": 3
}

num_to_move = {
    0: "up",
    1: "left",
    2: "down",
    3: "right"
}

# Start server when `python main.py` is run
if __name__ == "__main__":
    from server import run_server
    import sys

    if len(sys.argv) > 1:
        port = int(sys.argv[2])
        color = sys.argv[1]
        run_server({"info": info, "start": start, "move": move, "end": end, "port": port})
    else:
        run_server({"info": info, "start": start, "move": move, "end": end})
