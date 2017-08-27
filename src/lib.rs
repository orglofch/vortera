extern crate delaunay2d;
extern crate cgmath;
extern crate noise;
extern crate rand;

use cgmath::{InnerSpace, Point3, Vector3, Zero};
use delaunay2d::{Delaunay2D};
use noise::{Fbm, NoiseModule, Seedable};
use rand::Rng;
use std::collections::HashMap;

pub struct VoronoiTerrain {
    pub terrain_graph: Graph<TerrainVertex>,

    pub region_graph: Graph<Region>,

    pub water_level: u32,
}

impl VoronoiTerrain {
    pub fn builder() -> VoronoiTerrainBuilder {
        return VoronoiTerrainBuilder::new();
    }
}

pub struct VoronoiTerrainBuilder {
    seed: usize,
    water_level: u32,
    height: u32,
    sites: Vec<(f64, f64)>,
}

impl VoronoiTerrainBuilder {
    fn new() -> VoronoiTerrainBuilder {
        let mut rng = rand::thread_rng();
        VoronoiTerrainBuilder {
            seed: rng.gen::<usize>(),
            water_level: 50,
            height: 100,
            sites: Vec::new(),
        }
    }

    pub fn set_seed(&mut self, seed: usize) -> &mut VoronoiTerrainBuilder {
        self.seed = seed;
        self
    }

    pub fn set_sites(&mut self, sites: Vec<(f64, f64)>) -> &mut VoronoiTerrainBuilder {
        self.sites = sites;
        self
    }

    pub fn set_water_level(&mut self, water_level: u32) -> &mut VoronoiTerrainBuilder {
        self.water_level = water_level;
        self
    }

    pub fn set_height(&mut self, height: u32) -> &mut VoronoiTerrainBuilder {
        self.height = height;
        self
    }

    pub fn build(&self) -> VoronoiTerrain {
        // TODO(orglofch): Calculate the boundary from the sites.
        // TODO(orglofch): Port to Fortunes algorithm.
        let mut dt = Delaunay2D::new((0.0, 0.0), 9999.0);

        for site in self.sites.iter() {
            dt.add_point(*site);
        }

        let (dt_vertices, dt_cells) = dt.export_voronoi_regions();

        let noise = Fbm::new().set_seed(self.seed);

        // Generate the set of connecting edges for each vertex and the reverse for fast lookup.
        let mut terrain_edges_by_vertex_index: HashMap<usize, Vec<usize>> = HashMap::with_capacity(dt_vertices.len());

        // Capacity is a rough estimate of the total number of edges in the graph.
        // The average number of edges in a voronoi cell is < 6 and each is shared between 2 cells.
        let mut terrain_edges: Vec<(usize, usize)> = Vec::with_capacity(dt_vertices.len() * 2);
        let mut region_edges: Vec<(usize, usize)> = Vec::with_capacity(dt_vertices.len() * 2);

        // Terrain edges are near 1:1 with region edges (all except for the exterior cells).
        // TODO(orglofc): Consider combining the edges to a common structure so one can transition between the two.
        let mut region_by_terrain_edge: HashMap<(usize, usize), usize> = HashMap::with_capacity(dt_vertices.len() * 2);

        // For fast check retrieval of existing edges.
        let mut terrain_edge_index_by_edge: HashMap<(usize, usize), usize> = HashMap::with_capacity(dt_vertices.len() * 2);

        for (region_index, dt_cell) in dt_cells.iter().enumerate() {
            for (vertex_index, current_vertex_index) in dt_cell.iter().enumerate() {
                let next_vertex_index = dt_cell[(vertex_index + 1) % dt_cell.len()];

                // If the reverse edge already exists then we can skip adding this as a new edge.
                // The winding order will guarantee the duplicate edge is the reverse of the current edge.
                // We can also use the knowledge of when duplicates occur to mark region edges.
                let reverse_edge = (next_vertex_index, *current_vertex_index);

                let edge_index = match terrain_edge_index_by_edge.get(&reverse_edge) {
                    Some(&edge_index) => {
                        // Insert a new region edge.
                        let other_region = region_by_terrain_edge.get(&reverse_edge)
                            .expect("The reverse index should exist for the region index");
                        region_edges.push((region_index, *other_region));
                        edge_index
                    },
                    None => {
                        // Insert the new terrain edge.
                        let edge = (*current_vertex_index, next_vertex_index);
                        region_by_terrain_edge.insert(edge, region_index);
                        terrain_edge_index_by_edge.insert(edge, *current_vertex_index);
                        terrain_edges.push(edge);
                        terrain_edges.len() - 1
                    }
                };

                terrain_edges_by_vertex_index.entry(*current_vertex_index)
                    .or_insert(Vec::with_capacity(5)) // Average edges < 6.
                    .push(edge_index);
            }
        }

        // Create terrain vertices.
        let mut terrain_vertices: Vec<TerrainVertex> = Vec::with_capacity(dt_vertices.len());
        for (i, vertex) in dt_vertices.into_iter().enumerate() {
            let height = noise.get([vertex.0, vertex.1]);
            let terrain_vertex = TerrainVertex {
            position: Point3::new(vertex.0, vertex.1, height),
                normal: Vector3::zero(),
                edges: terrain_edges_by_vertex_index.remove(&i).unwrap(),
            };
            terrain_vertices.push(terrain_vertex);
        }

        // Create regions.
        let mut regions: Vec<Region> = Vec::with_capacity(dt_cells.len());

        for dt_cell in dt_cells.iter() {

            // Calculate the normal from 3 verices.
            let p0 = terrain_vertices[dt_cell[0]].position;
            let e1 = terrain_vertices[dt_cell[1]].position - p0;
            let e2 = terrain_vertices[dt_cell[2]].position - p0;
            let normal = e1.cross(e2);
            normal.normalize();

            let region = Region {
                center: Point3::new(0.0, 0.0, 0.0),
                normal: normal,
                edges: Vec::new(), // TODO(orglofch):
            };
            regions.push(region);
        }

        let terrain_graph = Graph {
            vertices: terrain_vertices,
            edges: terrain_edges,
        };

        let region_graph = Graph {
            vertices: regions,
            edges: region_edges,
        };

        VoronoiTerrain {
            terrain_graph: terrain_graph,
            region_graph: region_graph,
            water_level: self.water_level,
        }
    }
}

pub struct TerrainVertex {
    pub position: Point3<f64>,
    pub normal: Vector3<f64>,

    // Indices into the set of terrain_edges.
    pub edges: Vec<usize>,
}

pub struct Region {
    pub center: Point3<f64>,
    pub normal: Vector3<f64>,

    // Indices into the set of region_edges.
    pub edges: Vec<usize>,
}

pub struct Graph<T> {
    pub vertices: Vec<T>,
    pub edges: Vec<(usize, usize)>,
}
