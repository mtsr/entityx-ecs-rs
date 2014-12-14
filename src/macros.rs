// TODO allow with Player(1) style queries.

#[macro_export]
macro_rules! entities_with_components_inner(
    ( $em:ident, $already:expr : ) => ( $already );
    ( $em:ident, $already:expr : with $ty:path $( $kinds:ident $types:path )* ) => (
        entities_with_components_inner!( $em, $already.and_then(|tuple| {
            let comp = $em.get_component::<$ty>(&tuple.0);
            match comp {
                Some(obj) => Some( tuple.tup_append(obj) ),
                None => None
            }
        } ) : $( $kinds $types )* )
    );
    ( $em:ident, $already:expr : without $ty:path $( $kinds:ident $types:path )* ) => (
        entities_with_components_inner!( $em, $already.and_then(|tuple|
            if let &Some(_) = $em.get_component::<$ty>(&tuple.0) {
                None
            } else {
                Some(tuple)
            }
        ) : $( $kinds $types )* )
    );
    ( $em:ident, $already:expr : option $ty:path $( $kinds:ident $types:path )* ) => (
        entities_with_components_inner!( $em, $already.map(|tuple| {
            let comp = $em.get_component::<$ty>(&tuple.0);
            tuple.tup_append( comp )
        } ) : $( $kinds $types )* )
    );
)

#[macro_export]
macro_rules! entities_with_components(
    ( $em:ident : $( $kinds:ident $types:path )* ) => (
        $em.entities().filter_map(|entity|
            entities_with_components_inner!($em, Some((entity,)): $( $kinds $types )* )
        )
    );
)
