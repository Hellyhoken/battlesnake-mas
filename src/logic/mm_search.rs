use crate::{Board, /*Coord,*/ GameInfo};
use log::info;
use std::time::Instant;

// Define a tree node that can have many children
#[derive(Debug)]
struct TreeNode {
    value: i32,
    children: Vec<TreeNode>,
}

impl TreeNode {
    fn new(value: i32) -> Self {
        TreeNode {
            value,
            children: Vec::new(),
        }
    }

    fn add_child(&mut self, child: TreeNode) {
        //info!("Adding child with value: {}", child.value);
        self.children.push(child);
    }

    fn print(&self, prefix: String, is_last: bool) {
        println!(
            "{}{}─ {}",
            prefix,
            if is_last { "└" } else { "├" },
            self.value
        );
        let new_prefix = format!("{}{}", prefix, if is_last { "   " } else { "│  " });

        let last_index = self.children.len().saturating_sub(1);
        for (i, child) in self.children.iter().enumerate() {
            child.print(new_prefix.clone(), i == last_index);
        }
    }
}

use crate::logic::simple::SimpleBoard;

use super::simple::SnakeMove;

pub fn search(board: &Board, game_info: &GameInfo) -> [SnakeMove; 2] {
    let start = Instant::now();
    let simple_board = SimpleBoard::from(board, game_info);
    let timeout: i32 = game_info.timeout as i32 * 1_000_000; // Convert milliseconds to nanoseconds
    let mut values = Vec::new();
    let mut moves = Vec::new();

    let mut best_value = i32::MIN;
    let simulations = simple_board.simulate_move(true);
    for (i, (move_pair, next_board)) in simulations.iter().enumerate() {
        let time: i32 = (timeout - start.elapsed().as_nanos() as i32) / (simulations.len() as i32 - i as i32);
        info!("Move {} time: {} (timeout: {} elapsed: {})", i, time, timeout, start.elapsed().as_nanos());

        let mut root = TreeNode::new(0);

        // minmax on enemies since this outer loop is on friendly
        let value = minmax_simple(
            &next_board,
            1,
            false,
            best_value,
            i32::MAX,
            1,
            10,
            time,
            &mut root,
        ).0;
        //root.print(format!("{:?}:", move_pair), true);
        info!("Move {:?} value: {}", move_pair, value);
        best_value = best_value.max(value);
        values.push(value);
        moves.push(move_pair);
    }
    let idx = values
        .iter()
        .enumerate()
        .max_by(|(_, v), (_, v2)| v.cmp(v2))
        .map(|(i, _)| i)
        .expect(&format!(
            "No best move found in values: {:?} for {} moves",
            values,
            simulations.len()
        ));
    *moves[idx]
}

fn minmax_simple(
    board: &SimpleBoard,
    depth: i32,
    our_team: bool,
    mut alpha: i32,
    mut beta: i32,
    heuristic_time: i32,
    return_time: i32,
    timeout: i32,
    parent: &mut TreeNode,
) -> (i32, i32) {
    let start = Instant::now();
    let mut node = TreeNode::new(0);
    if depth == 100 || heuristic_time + return_time >= timeout {
        //info!("Depth {} reached", depth);
        let h = board.heuristic(false);
        node.value = h;
        parent.add_child(node);
        return (h, depth);
    }

    let mut simulations = board.simulate_move(our_team);
    if our_team {
        simulations.sort_by_key(|s| -s.1.heuristic(true));
    } else {
        simulations.sort_by_key(|s| s.1.heuristic(true));
    }

    if let Some(sim) = simulations.first() {
        let h = sim.1.heuristic(true);
        if our_team && h == i32::MAX {
            //info!("Found max value at depth {}", depth);
            node.value = i32::MAX;
            parent.add_child(node);
            return (i32::MAX, depth);
        } else if !our_team && h == i32::MIN {
            //info!("Found min value at depth {}", depth);
            node.value = i32::MIN;
            parent.add_child(node);
            return (i32::MIN, depth);
        }
    }

    let mut best_value = if our_team { (i32::MIN, depth) } else { (i32::MAX, depth)};

    for (idx, (_, next_board)) in simulations.iter().enumerate() {
        let time_left = timeout - start.elapsed().as_nanos() as i32 - return_time;
        if time_left <= heuristic_time {
            best_value = (simulations.first().unwrap().1.heuristic(false), depth+1);
            break;
        }

        let iterations_left = simulations.len() as i32 - idx as i32;
        let time_per_move = time_left / iterations_left;
        let value = minmax_simple(
            &next_board,
            depth + 1,
            !our_team,
            alpha,
            beta,
            heuristic_time,
            return_time,
            time_per_move,
            &mut node,
        );
        if our_team {
            if (value.0 > best_value.0) || (value.0 == best_value.0 && value.1 > best_value.1) {
                best_value = value;
                alpha = alpha.max(best_value.0);
                if best_value.0 >= beta {
                    break;
                }
            }
        } else {
            if (value.0 < best_value.0) || (value.0 == best_value.0 && value.1 > best_value.1) {
                best_value = value;
                beta = beta.max(best_value.0);
                if best_value.0 <= alpha {
                    break;
                }
            }
        }
    }

    //info!("Best value at depth {}: {}", depth, best_value);
    node.value = best_value.0;
    parent.add_child(node);
    if best_value.0 == i32::MAX {
        return best_value;
    }
    if best_value.0 == i32::MIN {
        return best_value;
    }
    let depth_diff = best_value.1 - depth;
    ((best_value.0 * depth_diff + board.heuristic(true)) / (depth_diff+1), best_value.1)
}
