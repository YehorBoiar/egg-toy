Each iteration works as follows:

```py
def canonicalize(enode)
    new_ch = [self.find(e) for e in enode.children]
    return mk_enode(enode.op, new_ch)

def add(enode):
    enode = self.canonicalize(enode)
    if enode in self.hashcons:
        return self.hashcons[enode]
    else:
        eclass_id = self.new_singleton_eclass(enode)
        for child in enode.children:
            child.parents.add(enode, eclass_id)
        self.hashcons[enode] = eclass_id
        return eclass_id

def merge(id1, id2)
    if self.find(id1) == self.find(id2):
        return self.find(id1)
    new_id = self.union_find.union(id1, id2)
    # traditional egraph merge can be
    # emulated by calling rebuild right after
    # adding the eclass to the worklist
    self.worklist.add(new_id)
    return new_id

def equality_saturation(expr, rewrites):
    egraph = initial_egraph(expr)
    while not egraph.is_saturated_or_timeout():
        matches = []
        # 1. Read-only phase (Search / Find)
        for rw in rewrites:
            for (subst, eclass) in egraph.ematch(rw.lhs):
                matches.append((rw, subst, eclass))

        # 2. Write-only phase (Apply)
        for (rw, subst, eclass) in matches:
            eclass2 = egraph.add(rw.rhs.subst(subst))
            egraph.merge(eclass, eclass2)

        # 3. Invariant restoration (Rebuild)
        egraph.rebuild()

    return egraph.extract_best()
```
