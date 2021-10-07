use std::ops::Deref;
use std::{cmp::max, collections::HashMap, hash, rc::Rc};


use std::collections::{self, HashSet};
use infer::dl;

use derive_more::*;
use LambdaCalcExpr::*;
use crate::utils::*;
use itertools::Itertools;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
enum LambdaCalcExpr{
   Ref(&'static str),
   Lam(&'static str, Rc<LambdaCalcExpr>),
   App(Rc<LambdaCalcExpr>, Rc<LambdaCalcExpr>)
}

impl LambdaCalcExpr {
   fn depth(&self) -> usize {
      match self{
         LambdaCalcExpr::Ref(_) => 0,
         LambdaCalcExpr::Lam(x,b) => 1 + b.depth(),
         LambdaCalcExpr::App(f,e) => 1 + max(f.depth(), e.depth())
      }
   }
}
fn app(f: LambdaCalcExpr, a: LambdaCalcExpr) -> LambdaCalcExpr {
   App(Rc::new(f), Rc::new(a))
}
fn lam(x: &'static str, e: LambdaCalcExpr) -> LambdaCalcExpr {
   Lam(x, Rc::new(e))
}

fn sub(exp: &LambdaCalcExpr, var: &str, e: &LambdaCalcExpr) -> LambdaCalcExpr {
   match exp {
      Ref(x) if *x == var => e.clone(),
      Ref(x) => exp.clone(),
      App(ef,ea) => app(sub(ef, var, e), sub(ea, var, e)),
      Lam(x, eb) if *x == var => exp.clone(),
      Lam(x, eb) => lam(x, sub(eb, var, e))
   }
}

fn U() -> LambdaCalcExpr {lam("x", app(Ref("x"), Ref("x")))}
fn I() -> LambdaCalcExpr {lam("x", Ref("x"))}

#[test]
fn test_dl_lambda(){
   dl!{
      relation output(LambdaCalcExpr);
      relation input(LambdaCalcExpr);
      relation eval(LambdaCalcExpr, LambdaCalcExpr);
      relation do_eval(LambdaCalcExpr);

      input(app(U(), I()));
      do_eval(exp.clone()) <-- input(exp);
      output(res.clone()) <-- input(exp), eval(exp, res);

      eval(exp.clone(), exp.clone()) <-- do_eval(?exp @Ref(_));

      eval(exp.clone(), exp.clone()) <-- do_eval(exp) if let Lam(_,_) = exp;

      do_eval(ef.as_ref().clone()) <-- do_eval(?App(ef,ea));

      do_eval(sub(fb, fx, ea)) <-- 
         do_eval(?App(ef, ea)), 
         eval(ef.deref(), ?Lam(fx, fb));
      
      eval(exp.clone(), final_res.clone()) <-- 
         do_eval(?exp @ App(ef, ea)), // this requires nightly
         eval(ef.deref(), ?Lam(fx, fb)),
         eval(sub(fb, fx, ea), final_res);
   };

   let mut prog = DLProgram::default();
   prog.run();   
   println!("output: {:?}", prog.output);
   assert!(prog.output.contains(&(I(),)));
   assert!(prog.output.len() == 1);
}

#[test]
fn test_dl_patterns(){
   dl!{
      relation foo(i32, Option<i32>);
      relation bar(i32, i32);
      foo(1, None);
      foo(2, Some(2));
      foo(3, Some(30));
      bar(*x, *y) <-- foo(x, y_opt) if let Some(y) = y_opt if y != x;
   };
   let mut prog = DLProgram::default();
   prog.run();
   println!("bar: {:?}", prog.bar);
   assert!(prog.bar.contains(&(3,30)));
   assert!(prog.bar.len() == 1);
}

#[test]
fn test_dl_pattern_args(){
   dl!{
      relation foo(i32, Option<i32>);
      relation bar(i32, i32);
      foo(1, None);
      foo(2, Some(2));
      foo(3, Some(30));
      foo(3, None);
      bar(*x, *y) <-- foo(x, ?Some(y)) if y != x;
   };
   let mut prog = DLProgram::default();
   prog.run();
   println!("bar: {:?}", prog.bar);
   assert!(prog.bar.contains(&(3,30)));
   assert!(prog.bar.len() == 1);
}

#[test]
fn test_dl2(){
   dl!{
      relation bar(i32, i32);
      relation foo1(i32, i32);
      relation foo2(i32, i32);

      foo1(1, 2);
      foo1(10, 20);
      foo1(0, 2);

      bar(*x, y + z) <-- foo1(x, y) if *x != 0, foo2(y, z);
   }

   let mut prog = DLProgram::default();
   
   let foo2 = vec![
      (2, 4),
      (2, 1),
      (20, 40),
      (20, 0),
   ];
   prog.foo2 = foo2;
   prog.update_indices();

   prog.run();

   println!("bar: {:?}", prog.bar);
   assert!(rels_equal([(1, 3), (1, 6), (10, 60), (10, 20)], prog.bar));
}

#[test]
fn test_dl_expressions(){
   dl!{
      relation foo(i32, i32);
      relation bar(i32, i32);
      relation baz(i32, i32, i32);
      foo(1, 2);
      foo(2, 3);
      foo(3, 5);

      bar(3, 6);
      bar(5, 10);

      baz(*x, *y, *z) <-- foo(x, y), bar(x + y , z);
   };
   let mut prog = DLProgram::default();
   prog.run();
   println!("baz: {:?}", prog.baz);
   assert!(rels_equal([(1,2,6), (2,3,10)], prog.baz));
}

#[test]
fn test_dl_vars_bound_in_patterns(){
   dl!{
      relation foo(i32, Option<i32>);
      relation bar(i32, i32);
      relation baz(i32, i32, i32);
      foo(1, Some(2));
      foo(2, None);
      foo(3, Some(5));
      foo(4, Some(10));


      bar(3, 6);
      bar(5, 10);
      bar(10, 20);

      baz(*x, *y, *z) <-- foo(x, ?Some(y)), bar(y , z);
   };
   let mut prog = DLProgram::default();
   prog.run();
   println!("baz: {:?}", prog.baz);
   assert!(rels_equal([(3, 5, 10), (4, 10, 20)], prog.baz));
}

#[test]
fn test_dl_generators(){
   dl!{
      relation foo(i32, i32);
      relation bar(i32);

      foo(x, y) <-- for x in 0..10, for y in (x+1)..10;

      bar(*x) <-- foo(x, y);
      bar(*y) <-- foo(x, y);
   };
   let mut prog = DLProgram::default();
   prog.run();
   println!("foo: {:?}", prog.foo);
   assert_eq!(prog.foo.len(), 45);
}

#[test]
fn test_dl_generators2(){
   dl!{
      relation foo(i32, i32);
      relation bar(i32);

      foo(3, 4);
      foo(4, 6);
      foo(20, 21);
      bar(*x) <-- for (x, y) in (0..10).map(|x| (x, x+1)), foo(x, y);

   };
   let mut prog = DLProgram::default();
   prog.run();
   println!("bar: {:?}", prog.bar);
   assert!(rels_equal([(3,)], prog.bar));
}


#[test]
fn test_dl_multiple_head_clauses(){
   dl!{
      relation foo(Vec<i32>, Vec<i32>);
      relation foo2(Vec<i32>);
      relation foo1(Vec<i32>);

      relation bar(i32);

      foo(vec![3], vec![4]);
      foo(vec![1, 2], vec![4, 5]);
      foo(vec![10, 11], vec![20]);

      foo1(x.clone()), foo2(y.clone()) <-- foo(x, y) if x.len() > 1;
   };
   let mut prog = DLProgram::default();
   prog.run();
   println!("foo1: {:?}", prog.foo1);
   println!("foo2: {:?}", prog.foo2);

   assert!(rels_equal([(vec![1, 2],), (vec![10,11],)], prog.foo1));
   assert!(rels_equal([(vec![4, 5],), (vec![20],)], prog.foo2));
}

#[test]
fn test_dl_multiple_head_clauses2(){
   dl!{
      relation foo(Vec<i32>);
      relation foo_left(Vec<i32>);
      relation foo_right(Vec<i32>);

      foo(vec![1,2,3,4]);

      foo_left(xs[..i].into()), foo_right(xs[i..].into()) <-- foo(xs), for i in 0..xs.len();
      foo(xs.clone()) <-- foo_left(xs);
      foo(xs.clone()) <-- foo_right(xs);
   };

   let mut prog = DLProgram::default();
   prog.run();
   println!("foo: {:?}", prog.foo);

   assert!(rels_equal([(vec![],), (vec![1],), (vec![1, 2],), (vec![1, 2, 3],), (vec![1, 2, 3, 4],), (vec![2],), (vec![2, 3],), (vec![2, 3, 4],), (vec![3],), (vec![3, 4],), (vec![4],)], 
                      prog.foo));
}
