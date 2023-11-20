use rand::Rng;

use super::{create_database, rng};
use crate::{Angular, Writer};

#[test]
fn write_one_vector_in_one_tree() {
    let handle = create_database();
    let mut wtxn = handle.env.write_txn().unwrap();
    let writer = Writer::<Angular>::prepare(&mut wtxn, handle.database, 3).unwrap();
    writer.add_item(&mut wtxn, 0, &[0.0, 1.0, 2.0]).unwrap();

    writer.build(&mut wtxn, rng(), Some(1)).unwrap();
    wtxn.commit().unwrap();

    insta::assert_display_snapshot!(handle, @r###"
    0: Leaf(Leaf { header: NodeHeaderAngular { norm: 2.236068 }, vector: [0.0, 1.0, 2.0] })
    1: Descendants(Descendants { descendants: [0] })

    u32::MAX: Metadata { dimensions: 3, root_nodes: [1] }
    "###);
}

#[test]
fn write_one_vector_in_multiple_trees() {
    let handle = create_database();
    let mut wtxn = handle.env.write_txn().unwrap();
    let writer = Writer::<Angular>::prepare(&mut wtxn, handle.database, 3).unwrap();
    writer.add_item(&mut wtxn, 0, &[0.0, 1.0, 2.0]).unwrap();

    writer.build(&mut wtxn, rng(), Some(10)).unwrap();
    wtxn.commit().unwrap();

    insta::assert_display_snapshot!(handle, @r###"
    0: Leaf(Leaf { header: NodeHeaderAngular { norm: 2.236068 }, vector: [0.0, 1.0, 2.0] })
    1: Descendants(Descendants { descendants: [0] })
    2: Descendants(Descendants { descendants: [0] })
    3: Descendants(Descendants { descendants: [0] })
    4: Descendants(Descendants { descendants: [0] })
    5: Descendants(Descendants { descendants: [0] })
    6: Descendants(Descendants { descendants: [0] })
    7: Descendants(Descendants { descendants: [0] })
    8: Descendants(Descendants { descendants: [0] })
    9: Descendants(Descendants { descendants: [0] })
    10: Descendants(Descendants { descendants: [0] })

    u32::MAX: Metadata { dimensions: 3, root_nodes: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] }
    "###);
}

#[test]
fn write_vectors_until_there_is_a_descendants() {
    let handle = create_database();
    let mut wtxn = handle.env.write_txn().unwrap();
    let writer = Writer::<Angular>::prepare(&mut wtxn, handle.database, 3).unwrap();
    for i in 0..3 {
        let id = i;
        let i = i as f32;
        writer.add_item(&mut wtxn, id, &[i, i, i]).unwrap();
    }

    writer.build(&mut wtxn, rng(), Some(1)).unwrap();
    wtxn.commit().unwrap();

    insta::assert_display_snapshot!(handle, @r###"
    0: Leaf(Leaf { header: NodeHeaderAngular { norm: 0.0 }, vector: [0.0, 0.0, 0.0] })
    1: Leaf(Leaf { header: NodeHeaderAngular { norm: 1.7320508 }, vector: [1.0, 1.0, 1.0] })
    2: Leaf(Leaf { header: NodeHeaderAngular { norm: 3.4641016 }, vector: [2.0, 2.0, 2.0] })
    3: Descendants(Descendants { descendants: [0, 1, 2] })

    u32::MAX: Metadata { dimensions: 3, root_nodes: [3] }
    "###);
}

#[test]
fn write_vectors_until_there_is_a_split() {
    let handle = create_database();
    let mut wtxn = handle.env.write_txn().unwrap();
    let writer = Writer::<Angular>::prepare(&mut wtxn, handle.database, 3).unwrap();
    for i in 0..4 {
        let id = i;
        let i = i as f32;
        writer.add_item(&mut wtxn, id, &[i, i, i]).unwrap();
    }

    writer.build(&mut wtxn, rng(), Some(1)).unwrap();
    wtxn.commit().unwrap();

    insta::assert_display_snapshot!(handle, @r###"
    0: Leaf(Leaf { header: NodeHeaderAngular { norm: 0.0 }, vector: [0.0, 0.0, 0.0] })
    1: Leaf(Leaf { header: NodeHeaderAngular { norm: 1.7320508 }, vector: [1.0, 1.0, 1.0] })
    2: Leaf(Leaf { header: NodeHeaderAngular { norm: 3.4641016 }, vector: [2.0, 2.0, 2.0] })
    3: Leaf(Leaf { header: NodeHeaderAngular { norm: 5.196152 }, vector: [3.0, 3.0, 3.0] })
    4: Descendants(Descendants { descendants: [1, 2, 3] })
    5: SplitPlaneNormal(SplitPlaneNormal { normal: [0.57735026, 0.57735026, 0.57735026], left: 0, right: 4 })

    u32::MAX: Metadata { dimensions: 3, root_nodes: [5] }
    "###);
}

#[test]
fn write_a_lot_of_random_points() {
    let handle = create_database();
    let mut wtxn = handle.env.write_txn().unwrap();
    let writer = Writer::<Angular>::prepare(&mut wtxn, handle.database, 30).unwrap();
    let mut rng = rng();
    for id in 0..100 {
        let vector: [f32; 30] = std::array::from_fn(|_| rng.gen());
        writer.add_item(&mut wtxn, id, &vector).unwrap();
    }

    writer.build(&mut wtxn, rng, Some(10)).unwrap();
    wtxn.commit().unwrap();

    insta::assert_display_snapshot!(handle);
}
