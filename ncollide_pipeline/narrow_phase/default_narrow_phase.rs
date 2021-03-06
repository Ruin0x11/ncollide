use std::collections::HashMap;
use std::collections::hash_map::Entry;

use utils::data::SortedPair;
use geometry::query::Proximity;
use narrow_phase::{ContactAlgorithm, ContactDispatcher, ContactPairs, NarrowPhase,
                   ProximityAlgorithm, ProximityDispatcher, ProximityPairs};
use world::{CollisionObjectHandle, CollisionObjectSlab, GeometricQueryType};
use events::{ContactEvent, ContactEvents, ProximityEvent, ProximityEvents};
use math::Point;

// FIXME: move this to the `narrow_phase` module.
/// Collision detector dispatcher for collision objects.
pub struct DefaultNarrowPhase<P, M> {
    contact_dispatcher: Box<ContactDispatcher<P, M>>,
    contact_generators: HashMap<SortedPair<CollisionObjectHandle>, ContactAlgorithm<P, M>>,

    proximity_dispatcher: Box<ProximityDispatcher<P, M>>,
    proximity_detectors: HashMap<SortedPair<CollisionObjectHandle>, ProximityAlgorithm<P, M>>,
}

impl<P: Point, M: 'static> DefaultNarrowPhase<P, M> {
    /// Creates a new `DefaultNarrowPhase`.
    pub fn new(
        contact_dispatcher: Box<ContactDispatcher<P, M>>,
        proximity_dispatcher: Box<ProximityDispatcher<P, M>>,
    ) -> DefaultNarrowPhase<P, M> {
        DefaultNarrowPhase {
            contact_dispatcher: contact_dispatcher,
            contact_generators: HashMap::new(),

            proximity_dispatcher: proximity_dispatcher,
            proximity_detectors: HashMap::new(),
        }
    }
}

impl<P: Point, M: 'static, T> NarrowPhase<P, M, T> for DefaultNarrowPhase<P, M> {
    fn update(
        &mut self,
        objects: &CollisionObjectSlab<P, M, T>,
        contact_events: &mut ContactEvents,
        proximity_events: &mut ProximityEvents,
        timestamp: usize,
    ) {
        for (key, value) in self.contact_generators.iter_mut() {
            let co1 = &objects[key.0];
            let co2 = &objects[key.1];

            if co1.timestamp == timestamp || co2.timestamp == timestamp {
                let had_contacts = value.num_contacts() != 0;

                if let Some(prediction) = co1.query_type().contact_queries_to_prediction(co2.query_type()) {
                    let _ = value.update(
                        &*self.contact_dispatcher,
                        &co1.position(),
                        co1.shape().as_ref(),
                        &co2.position(),
                        co2.shape().as_ref(),
                        &prediction,
                    );
                } else {
                    panic!("Unable to compute contact between collision objects with query types different from `GeometricQueryType::Contacts(..)`.")
                } 

                if value.num_contacts() == 0 {
                    if had_contacts {
                        contact_events.push(ContactEvent::Stopped(co1.handle(), co2.handle()));
                    }
                } else {
                    if !had_contacts {
                        contact_events.push(ContactEvent::Started(co1.handle(), co2.handle()));
                    }
                }
            }
        }

        for (key, value) in self.proximity_detectors.iter_mut() {
            let co1 = &objects[key.0];
            let co2 = &objects[key.1];

            if co1.timestamp == timestamp || co2.timestamp == timestamp {
                let prev_prox = value.proximity();

                let _ = value.update(
                    &*self.proximity_dispatcher,
                    &co1.position(),
                    co1.shape().as_ref(),
                    &co2.position(),
                    co2.shape().as_ref(),
                    co1.query_type().query_limit() + co2.query_type().query_limit(),
                );

                let new_prox = value.proximity();

                if new_prox != prev_prox {
                    proximity_events.push(ProximityEvent::new(
                        co1.handle(),
                        co2.handle(),
                        prev_prox,
                        new_prox,
                    ));
                }
            }
        }
    }

    fn handle_interaction(
        &mut self,
        contact_events: &mut ContactEvents,
        proximity_events: &mut ProximityEvents,
        objects: &CollisionObjectSlab<P, M, T>,
        handle1: CollisionObjectHandle,
        handle2: CollisionObjectHandle,
        started: bool,
    ) {
        let key = SortedPair::new(handle1, handle2);
        let co1 = &objects[handle1];
        let co2 = &objects[handle2];

        match (co1.query_type(), co2.query_type()) {
            (GeometricQueryType::Contacts(..), GeometricQueryType::Contacts(..)) => {
                if started {
                    let dispatcher = &self.contact_dispatcher;

                    if let Entry::Vacant(entry) = self.contact_generators.entry(key) {
                        if let Some(detector) = dispatcher
                            .get_contact_algorithm(co1.shape().as_ref(), co2.shape().as_ref())
                        {
                            let _ = entry.insert(detector);
                        }
                    }
                } else {
                    // Proximity stopped.
                    if let Some(detector) = self.contact_generators.remove(&key) {
                        // Register a collision lost event if there was a contact.
                        if detector.num_contacts() != 0 {
                            contact_events.push(ContactEvent::Stopped(co1.handle(), co2.handle()));
                        }
                    }
                }
            }
            (_, GeometricQueryType::Proximity(_)) | (GeometricQueryType::Proximity(_), _) => {
                if started {
                    let dispatcher = &self.proximity_dispatcher;

                    if let Entry::Vacant(entry) = self.proximity_detectors.entry(key) {
                        if let Some(detector) = dispatcher
                            .get_proximity_algorithm(co1.shape().as_ref(), co2.shape().as_ref())
                        {
                            let _ = entry.insert(detector);
                        }
                    }
                } else {
                    // Proximity stopped.
                    if let Some(detector) = self.proximity_detectors.remove(&key) {
                        // Register a proximity lost signal if they were not disjoint.
                        let prev_prox = detector.proximity();

                        if prev_prox != Proximity::Disjoint {
                            let event = ProximityEvent::new(
                                co1.handle(),
                                co2.handle(),
                                prev_prox,
                                Proximity::Disjoint,
                            );
                            proximity_events.push(event);
                        }
                    }
                }
            }
        }
    }

    fn handle_removal(
        &mut self,
        _: &CollisionObjectSlab<P, M, T>,
        handle1: CollisionObjectHandle,
        handle2: CollisionObjectHandle,
    ) {
        let key = SortedPair::new(handle1, handle2);
        let _ = self.proximity_detectors.remove(&key);
        let _ = self.contact_generators.remove(&key);
    }

    fn contact_pairs<'a>(
        &'a self,
        objects: &'a CollisionObjectSlab<P, M, T>,
    ) -> ContactPairs<'a, P, M, T> {
        ContactPairs::new(objects, self.contact_generators.iter())
    }

    fn proximity_pairs<'a>(
        &'a self,
        objects: &'a CollisionObjectSlab<P, M, T>,
    ) -> ProximityPairs<'a, P, M, T> {
        ProximityPairs::new(objects, self.proximity_detectors.iter())
    }
}
