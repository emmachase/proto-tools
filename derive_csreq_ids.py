from binaryninja import *

warnings = 0
errors = 0

def main(bv: BinaryView):
    global warnings, errors

    fns = bv.get_functions_by_name("RPG.Client.NetworkManager::SendPacket")
    if len(fns) == 0:
        show_message_box("Error", "Couldn't find SendPacket function, please annotate it", MessageBoxButtonSet.OKButtonSet, MessageBoxIcon.ErrorIcon)
        return
    
    send_packet_fn = fns[0]

    caller_sites = send_packet_fn.caller_sites

    # all_names = set([type(x.hlil.ssa_form).__name__ for x in caller_sites])
    # print(all_names)
    count = 0

    for cs in caller_sites:
        # print(cs.hlil)
        ssa = cs.hlil.ssa_form
        # print([ssa])
        # print(ssa.vars)
        # # print python class name
        # print(type(ssa).__name__)

        def check(instr: HighLevelILInstruction):
            # print(type(instr))
            if type(instr) == HighLevelILCallSsa or type(instr) == HighLevelILTailcall:
                return instr
            return None
                
        calls = [x for x in ssa.traverse(check) if x is not None]
        if len(calls) != 1:
            print(calls)
            show_message_box("Warning", "Unexpected number of calls in SendPacket", MessageBoxButtonSet.OKButtonSet, MessageBoxIcon.WarningIcon)
            warnings += 1
            continue

        call = calls[0]
        # print(call.params)

        cmdid = call.params[1]
        if type(cmdid) != HighLevelILConst:
            print("Warning: cmdid is not a constant, cannot extract from ref: ", ssa, hex(ssa.address))
            warnings += 1
            continue
            # return None, None, True

        def traverse_calls(calls, param_index=2):
            global warnings, errors

            call = calls[0]

            if len(call.params) == 0:
                print("Error: call params is empty, cannot extract from ref: ", ssa, hex(ssa.address))
                errors += 1
                return None, True

            data = call.params[param_index]
            if type(data) != HighLevelILVarSsa:
                print("Warning: data is not a variable, cannot extract from ref: ", ssa, hex(ssa.address))
                warnings += 1
                return None, True

            # print(cmdid.value, [data.var.def_site])
            def_site = data.var.def_site
            if def_site is None:

                if data.var.var.is_parameter_variable:
                    # Recurse to find the actual def_site
                    fn = data.var.function
                    index = list(fn.parameter_vars).index(data.var.var)
                    
                    for caller_fn in fn.caller_sites:
                        caller_ssa = caller_fn.hlil.ssa_form

                        ns_calls = [x for x in caller_ssa.traverse(check) if x is not None]
                        if len(ns_calls) != 1:
                            print(ns_calls, fn)
                            show_message_box("Warning", "Unexpected number of ns_calls in Nested Function Call", MessageBoxButtonSet.OKButtonSet, MessageBoxIcon.WarningIcon)
                            warnings += 1
                            continue

                        print("recurse", caller_ssa, hex(caller_ssa.address))
                        cfn, err = traverse_calls(ns_calls, param_index=index)
                        if err:
                            continue

                        return cfn, False
                    else:
                        print("Error: no call sites matched, cannot extract from ref: ", ssa, hex(ssa.address))
                        errors += 1
                        return None, True

                print("Error: def_site is not defined, cannot extract from ref: ", ssa, hex(ssa.address))
                warnings += 1
                return None, True

            # print([def_site])
            if type(def_site) != HighLevelILVarInitSsa:
                print("Error: def_site is not var-init, cannot extract from ref: ", ssa, hex(ssa.address))
                warnings += 1
                return None, True

            alloc_src = def_site.src
            if type(alloc_src) != HighLevelILCallSsa:
                print("Error: alloc_src is not a call, cannot extract from ref: ", ssa, hex(ssa.address))
                errors += 1
                return None, True
            
            alloc_params = alloc_src.params
            # def_site: HighLevelILInstruction
            # alloc_params = def_site.src.params
            if len(alloc_params) != 1:
                print("Error: alloc_params is not 1, cannot extract from ref: ", ssa, hex(ssa.address))
                errors += 1
                return None, True
            
            alloc_param = alloc_params[0]
            # print(type(alloc_param))

            if type(alloc_param) == HighLevelILDerefSsa:
                pass
            elif type(alloc_param) == HighLevelILVarSsa:
                # print(alloc_param.var.def_site.src)
                # print(type(alloc_param.var.def_site.src))
                def_site = alloc_param.var.def_site
                if def_site is None:
                    print("Error: def_site is not defined of alloc var, cannot extract from ref: ", ssa, hex(ssa.address))
                    errors += 1
                    return None, True

                alloc_param = def_site.src

                if type(alloc_param) != HighLevelILDerefSsa:
                    print("Error: alloc_param is not deref, cannot extract from ref: ", ssa, hex(ssa.address))
                    errors += 1
                    return None, True
                # raise NotImplementedError
            else:
                print("Error: alloc_param is not deref or var, cannot extract from ref: ", ssa, hex(ssa.address))
                errors += 1
                return None, True

            # print(alloc_params, ssa, hex(ssa.address))
            if type(alloc_param.src) != HighLevelILConstPtr:
                print("Error: alloc_param.src is not constptr, cannot extract from ref: ", ssa, hex(ssa.address))
                errors += 1
                return None, True

            il2cpp_class_ref = alloc_param.src.constant
            class_refs = bv.get_code_refs(il2cpp_class_ref)
            
            # Find ::Clone ref
            for ref in class_refs:
                if ref.function.name.endswith("::Clone"):
                    clone_fn = ref.function
                    break
            else:
                print("Error: Couldn't find ::Clone ref, cannot extract from ref: ", ssa, hex(ssa.address))
                errors += 1
                return None, True

            return clone_fn, False
        
        clone_fn, err = traverse_calls(calls)
        if err:
            continue

        nameTranslation = clone_fn.name.split("::")[0]
        print(f"{nameTranslation} => {cmdid.value.value}")

        count += 1
    # ss[1].hlil.ssa_form.vars[1].def_site.src.params[0].src.constant


    # print(send_packet_fn)
    
if __name__ == "__main__":
    bv: BinaryView
    main(bv)
