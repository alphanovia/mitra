use mitra_core::model::*;
fn p(n:&str)->Piece{Piece::named(n)}
fn main(){
 for (pass,fail) in [(0,1),(1,1),(2,2),(10,1),(100,1),(10,10),(0,4)]{
  let mut m=Model::default();
  m.base.push((p("gravity"),0.5));
  let n=pass+fail;
  for k in 0..n{ let fell=k<pass; let (mass,same,drop)=(format!("mass {k}"),format!("same speed {k}"),format!("drop {k}"));
   let rel=p(&format!("all: {same}"));
   m.related.push(Statement{piece:rel.clone(),content:Related{how:Relation::All,pieces:vec![p(&same),p("gravity"),p(&mass)]}});
   m.base.push((rel,1.0)); m.base.push((p(&mass),0.5)); m.base.push((p(&drop),0.9));
   m.evidence.push(Evidence{piece:p(&drop),about:p(&same),value:if fell{yes()}else{no()},by:Voice::named(&drop)});}
  let t=std::time::Instant::now();
  let show=|x:&str|{let o=opinion(&m,&p(x)); format!("{x} ({:.2}, {:.3}) P{:.3}",o.r,o.s,projected(o))};
  let last=n-1;
  println!("pass {pass} fail {fail}: {} | {} | {} | {} | {:?}",show("gravity"),show("mass 0"),show(&format!("mass {last}")),show(&format!("drop {last}")), t.elapsed());
  if n<=4 {println!("   {:?}",conflicts(&m).iter().map(|g|g.iter().map(|x|x.name.clone()).collect::<Vec<_>>()).collect::<Vec<_>>());}
 }
}
