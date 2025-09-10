import React from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import AuthLayout from "../../layouts/AuthLayout";
import Input from "../../components/Input";
import Button from "../../components/Button";
import { Link } from "react-router-dom";

const RegisterSchema = z
  .object({
    email: z
      .string()
      .email("Invalid email address")
      .min(1, "Email is required"),
    password: z.string().min(6, "Password must be at least 6 characters"),
    confirmPassword: z.string().min(1, "Confirm Password is required"),
  })
  .refine((data) => data.password === data.confirmPassword, {
    message: "Password don't match",
    path: ["confirmPassword"],
  });

type RegisterFormInputs = z.infer<typeof RegisterSchema>;

const Register: React.FC = () => {
  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<RegisterFormInputs>({
    resolver: zodResolver(RegisterSchema),
  });
  const onSubmit = async (data: RegisterFormInputs) => {
    console.log("Register attempt:", data);
    await new Promise((resolve) => setTimeout(resolve, 1000));
    alert("Registration submitted! Check console for values.");
  };

  return (
    <AuthLayout
      title="Create your account"
      subtitle="Start your journey with us "
    >
      <form className="mt-8 space-y-6" onSubmit={handleSubmit(onSubmit)}>
        <Input
          id="email"
          label="Email address"
          type="email"
          placeholder="Enter your email"
          {...register("email")}
          error={errors.email}
        />
        <Input
          id="password"
          label="Password"
          type="password"
          placeholder="Create a password"
          {...register("password")}
          error={errors.password}
        />
        <Input
          id="confirmPassword"
          label="Confirm Password"
          type="password"
          placeholder="Confirm your password"
          {...register("confirmPassword")}
          error={errors.confirmPassword}
        />

        <div>
          <Button type="submit" disabled={isSubmitting} className="w-full">
            {isSubmitting ? "Registering..." : "Register"}
          </Button>
        </div>
        <div className="text-center text-sm text-gray-600">
          Already have an account?{" "}
          <Link
            to="/login"
            className="font-medium text-blue-600 hover:text-blue-500"
          >
            Sign In
          </Link>
        </div>
      </form>
    </AuthLayout>
  );
};
export default Register;
