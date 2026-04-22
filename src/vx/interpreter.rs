use crate::vx::ast::*;
use crate::scene::{Scene, Mesh, Material, create_cube, create_sphere, create_cylinder, create_plane};
use crate::math::{Vec3, Quat};
use std::collections::HashMap;

pub struct Interpreter {
    variables: HashMap<String, GeometryValue>,
}

#[allow(dead_code)]
#[derive(Clone)]
enum GeometryValue {
    Mesh(Mesh),
    Material(Material),
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn execute(&mut self, program: &Program, scene: &mut Scene) -> Result<(), String> {
        for statement in &program.statements {
            self.execute_statement(statement, scene)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, statement: &Statement, scene: &mut Scene) -> Result<(), String> {
        match statement {
            Statement::Let { name, value } => {
                let mesh = self.eval_expr(value)?;
                self.variables.insert(name.clone(), GeometryValue::Mesh(mesh));
                Ok(())
            }
            Statement::Spawn(expr) => {
                let mesh = self.eval_expr(expr)?;
                let id = scene.create_entity();
                if let Some(entity) = scene.get_mut(id) {
                    entity.mesh = Some(mesh);
                }
                Ok(())
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Mesh, String> {
        match expr {
            Expr::Primitive(prim) => self.eval_primitive(prim),
            Expr::Variable(name) => {
                match self.variables.get(name) {
                    Some(GeometryValue::Mesh(mesh)) => Ok(mesh.clone()),
                    _ => Err(format!("Variable '{}' not found or not a mesh", name)),
                }
            }
            Expr::MethodCall { object, method, args } => {
                let mut mesh = self.eval_expr(object)?;
                self.apply_method(&mut mesh, method, args)?;
                Ok(mesh)
            }
            Expr::FunctionCall { name, args } => {
                self.eval_function(name, args)
            }
        }
    }

    fn eval_primitive(&self, prim: &Primitive) -> Result<Mesh, String> {
        match prim {
            Primitive::Cube { width, height, depth } => {
                Ok(create_cube(*width, *height, *depth))
            }
            Primitive::Sphere { radius, segments, rings } => {
                Ok(create_sphere(*radius, *segments, *rings))
            }
            Primitive::Cylinder { radius, height, segments } => {
                Ok(create_cylinder(*radius, *height, *segments))
            }
            Primitive::Plane { width, depth } => {
                Ok(create_plane(*width, *depth))
            }
        }
    }

    fn apply_method(&self, mesh: &mut Mesh, method: &str, args: &[Argument]) -> Result<(), String> {
        match method {
            "translate" => {
                let x = self.get_arg_f32(args, "x").unwrap_or(0.0);
                let y = self.get_arg_f32(args, "y").unwrap_or(0.0);
                let z = self.get_arg_f32(args, "z").unwrap_or(0.0);
                let translation = glam::Mat4::from_translation(Vec3::new(x, y, z));
                mesh.transform(&translation);
                Ok(())
            }
            "scale" => {
                let x = self.get_arg_f32(args, "x").unwrap_or(1.0);
                let y = self.get_arg_f32(args, "y").unwrap_or(1.0);
                let z = self.get_arg_f32(args, "z").unwrap_or(1.0);
                let scale = glam::Mat4::from_scale(Vec3::new(x, y, z));
                mesh.transform(&scale);
                Ok(())
            }
            "rotate" => {
                let x = self.get_arg_f32(args, "x").unwrap_or(0.0);
                let y = self.get_arg_f32(args, "y").unwrap_or(0.0);
                let z = self.get_arg_f32(args, "z").unwrap_or(0.0);
                let rotation = Quat::from_euler(glam::EulerRot::XYZ, x, y, z);
                let rotation_mat = glam::Mat4::from_quat(rotation);
                mesh.transform(&rotation_mat);
                Ok(())
            }
            "set_material" => {
                Ok(())
            }
            _ => Err(format!("Unknown method: {}", method)),
        }
    }

    fn eval_function(&mut self, name: &str, args: &[Expr]) -> Result<Mesh, String> {
        match name {
            "union" | "difference" | "intersection" => {
                if args.len() != 2 {
                    return Err(format!("{} requires exactly 2 arguments", name));
                }
                let mesh1 = self.eval_expr(&args[0])?;
                let mesh2 = self.eval_expr(&args[1])?;
                self.csg_operation(name, mesh1, mesh2)
            }
            _ => Err(format!("Unknown function: {}", name)),
        }
    }

    fn csg_operation(&self, _op: &str, mesh1: Mesh, _mesh2: Mesh) -> Result<Mesh, String> {
        Ok(mesh1)
    }

    fn get_arg_f32(&self, args: &[Argument], name: &str) -> Option<f32> {
        args.iter()
            .find(|arg| arg.name == name)
            .and_then(|arg| arg.value.as_f32())
    }
}
