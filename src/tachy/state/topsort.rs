// +--------------------------------------------------------------------------+
// | Copyright 2018 Matthew D. Steele <mdsteele@alum.mit.edu>                 |
// |                                                                          |
// | This file is part of Tachyomancer.                                       |
// |                                                                          |
// | Tachyomancer is free software: you can redistribute it and/or modify it  |
// | under the terms of the GNU General Public License as published by the    |
// | Free Software Foundation, either version 3 of the License, or (at your   |
// | option) any later version.                                               |
// |                                                                          |
// | Tachyomancer is distributed in the hope that it will be useful, but      |
// | WITHOUT ANY WARRANTY; without even the implied warranty of               |
// | MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU        |
// | General Public License for details.                                      |
// |                                                                          |
// | You should have received a copy of the GNU General Public License along  |
// | with Tachyomancer.  If not, see <http://www.gnu.org/licenses/>.          |
// +--------------------------------------------------------------------------+

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::mem;

//===========================================================================//

pub fn topological_sort_into_groups<N, FN, IN>(
    nodes: &[N], mut successors: FN)
    -> Result<Vec<Vec<N>>, (Vec<Vec<N>>, Vec<N>)>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    if nodes.is_empty() {
        return Ok(Vec::new());
    }
    let mut succs_map = HashMap::<N, HashSet<N>>::with_capacity(nodes.len());
    let mut preds_map = HashMap::<N, usize>::with_capacity(nodes.len());
    for node in nodes.iter() {
        succs_map.insert(node.clone(), successors(node).into_iter().collect());
        preds_map.insert(node.clone(), 0);
    }
    for succs in succs_map.values() {
        for succ in succs.iter() {
            *preds_map.get_mut(succ).unwrap() += 1;
        }
    }
    let mut groups = Vec::<Vec<N>>::new();
    let mut prev_group: Vec<N> = preds_map
        .iter()
        .filter_map(|(node, &num_preds)| if num_preds == 0 {
                        Some(node.clone())
                    } else {
                        None
                    })
        .collect();
    if prev_group.is_empty() {
        let remaining: Vec<N> =
            preds_map.into_iter().map(|(node, _)| node).collect();
        return Err((Vec::new(), remaining));
    }
    for node in prev_group.iter() {
        preds_map.remove(node);
    }
    while !preds_map.is_empty() {
        let mut next_group = Vec::<N>::new();
        for node in prev_group.iter() {
            for succ in succs_map.get(node).unwrap() {
                {
                    let num_preds = preds_map.get_mut(succ).unwrap();
                    *num_preds -= 1;
                    if *num_preds > 0 {
                        continue;
                    }
                }
                next_group.push(preds_map.remove_entry(succ).unwrap().0);
            }
        }
        groups.push(mem::replace(&mut prev_group, next_group));
        if prev_group.is_empty() {
            let remaining: Vec<N> =
                preds_map.into_iter().map(|(node, _)| node).collect();
            return Err((groups, remaining));
        }
    }
    groups.push(prev_group);
    Ok(groups)
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::topological_sort_into_groups;

    // Wrapper around topological_sort_into_groups that sorts each group.
    fn topsort(succs: &[&[usize]])
               -> Result<Vec<Vec<usize>>, (Vec<Vec<usize>>, Vec<usize>)> {
        let nodes: Vec<usize> = (0..succs.len()).into_iter().collect();
        match topological_sort_into_groups(&nodes, |&n| {
            succs[n].iter().cloned()
        }) {
            Ok(mut groups) => {
                for group in groups.iter_mut() {
                    group.sort();
                }
                Ok(groups)
            }
            Err((mut groups, mut remaining)) => {
                for group in groups.iter_mut() {
                    group.sort();
                }
                remaining.sort();
                Err((groups, remaining))
            }
        }
    }

    #[test]
    fn empty_graph() {
        assert_eq!(topsort(&[]), Ok(vec![]));
    }

    #[test]
    fn graph_with_no_edges() {
        assert_eq!(topsort(&[&[], &[], &[]]), Ok(vec![vec![0, 1, 2]]));
    }

    #[test]
    fn diamond() {
        assert_eq!(topsort(&[&[1, 2], &[3], &[3], &[]]),
                   Ok(vec![vec![0], vec![1, 2], vec![3]]));
    }

    #[test]
    fn multiple_layers() {
        let succs: &[&[usize]] = &[&[1, 5], &[2], &[3], &[], &[5], &[3]];
        assert_eq!(topsort(succs),
                   Ok(vec![vec![0, 4], vec![1, 5], vec![2], vec![3]]));
    }

    #[test]
    fn nothing_but_a_cycle() {
        assert_eq!(topsort(&[&[1], &[2], &[0]]), Err((vec![], vec![0, 1, 2])));
    }

    #[test]
    fn chain_then_cycle() {
        assert_eq!(topsort(&[&[1], &[2], &[3], &[2, 4], &[]]),
                   Err((vec![vec![0], vec![1]], vec![2, 3, 4])));
    }

    #[test]
    fn self_edge() {
        assert_eq!(topsort(&[&[1, 2], &[3], &[3], &[3]]),
                   Err((vec![vec![0], vec![1, 2]], vec![3])));
    }
}

//===========================================================================//
