.altmacro

.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm

.macro SAVE_GP_RANGE start, end
    .set n, start
    .rept end - start + 1
        SAVE_GP %n
        .set n, n + 1
    .endr
.endm

.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm

.macro LOAD_GP_RANGE start, end
    .set n, start
    .rept end - start + 1
        LOAD_GP %n
        .set n, n + 1
    .endr
.endm

    .section .text
    .globl __trap_from_user
    .globl __return_to_user
    .globl __trap_from_kernel
    .align 2


# user -> kernel
__trap_from_user:
    # sp <-> sscratch , 用户栈顶保存在sscratch
    csrrw sp, sscratch, sp
    
    # 现在sp 指向 TrapContext， 而sscratch指向用户栈顶
    # 保存TrapContext中的寄存器x1
    # sd x1, 1*8(sp)
    SAVE_GP 1

    # 这里跳过保存x2，而是从x3 - x31
    # 后面会保存x2
    # 这里是一个循环+宏
    SAVE_GP_RANGE 3, 31

    # 在上一行已经保存过t0-t2,现在可以使用
    # 保存sstatus sepc
    csrr t0, sstatus
    csrr t1, sepc

    # 将sstatus sepc放入创建的TrapContext对应位置中
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)

    # sscratch中是用户sp，将sp存入TrapContext对应位置中
    csrr t2, sscratch
    sd t2, 2*8(sp)

    # 解下来讲进入kernel， 所以要加载对应寄存器
    # # move to kernel_sp
    # 保存kernel的返回地址
    ld ra, 35*8(sp)

    # load callee-saved regs
    # s0 - s11
    ld s0, 36*8(sp)
    ld s1, 37*8(sp)
    ld s2, 38*8(sp)
    ld s3, 39*8(sp)
    ld s4, 40*8(sp)
    ld s5, 41*8(sp)
    ld s6, 42*8(sp)
    ld s7, 43*8(sp)
    ld s8, 44*8(sp)
    ld s9, 45*8(sp)
    ld s10, 46*8(sp)
    ld s11, 47*8(sp)

    # load kernel fp tp
    ld fp, 48*8(sp)
    ld tp, 49*8(sp)

    # 最后加载kernel的栈顶指针
    ld sp, 34*8(sp)

    # return to kernel ra
    ret

# kernel -> user
__return_to_user:
    # 此时a0是TrapContext指针
    # switch to user space

    # 更新sscratch，让sscratch中再次存入TrapContext地址，方便下次使用
    csrw sscratch, a0

    # 保存kernel的寄存器
    # 这里对应上面trap_from_user的一系列ld kernel寄存器
    sd sp, 34*8(a0)  # 保存栈顶指针
    sd ra, 35*8(a0)  # 保存ra，返回地址
    sd s0, 36*8(a0)  # 保存s1 - s11
    sd s1, 37*8(a0)
    sd s2, 38*8(a0)
    sd s3, 39*8(a0)
    sd s4, 40*8(a0)
    sd s5, 41*8(a0)
    sd s6, 42*8(a0)
    sd s7, 43*8(a0)
    sd s8, 44*8(a0)
    sd s9, 45*8(a0)
    sd s10, 46*8(a0)
    sd s11, 47*8(a0)
    sd fp, 48*8(a0)  # 保存fp
    sd tp, 49*8(a0)  # 保存tp

    # 将栈顶指向TrapContext
    mv sp, a0

    # now sp points to TrapContext in kernel space, start restoring based on it
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1

    # 恢复用户寄存器， x0和sp在后面单独恢复
    ld x1, 1*8(sp)
    LOAD_GP_RANGE 3, 31

    # 恢复用户栈顶指针
    ld sp, 2*8(sp)

    # 返回到用户
    sret

# 处理内核自己的trap
# kernel -> kernel
__trap_from_kernel:
    # only need to save caller-saved regs
    # note that we don't save sepc & stvec here
    addi sp, sp, -17*8
    sd  ra,  1*8(sp)
    sd  t0,  2*8(sp)
    sd  t1,  3*8(sp)
    sd  t2,  4*8(sp)
    sd  t3,  5*8(sp)
    sd  t4,  6*8(sp)
    sd  t5,  7*8(sp)
    sd  t6,  8*8(sp)
    sd  a0,  9*8(sp)
    sd  a1, 10*8(sp)
    sd  a2, 11*8(sp)
    sd  a3, 12*8(sp)
    sd  a4, 13*8(sp)
    sd  a5, 14*8(sp)
    sd  a6, 15*8(sp)
    sd  a7, 16*8(sp)
    call kernel_trap_handler
    ld  ra,  1*8(sp)
    ld  t0,  2*8(sp)
    ld  t1,  3*8(sp)
    ld  t2,  4*8(sp)
    ld  t3,  5*8(sp)
    ld  t4,  6*8(sp)
    ld  t5,  7*8(sp)
    ld  t6,  8*8(sp)
    ld  a0,  9*8(sp)
    ld  a1, 10*8(sp)
    ld  a2, 11*8(sp)
    ld  a3, 12*8(sp)
    ld  a4, 13*8(sp)
    ld  a5, 14*8(sp)
    ld  a6, 15*8(sp)
    ld  a7, 16*8(sp)
    addi sp, sp, 17*8
    sret
