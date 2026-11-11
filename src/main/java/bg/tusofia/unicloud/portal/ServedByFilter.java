package bg.tusofia.unicloud.portal;

import java.io.IOException;

import jakarta.servlet.FilterChain;
import jakarta.servlet.ServletException;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.springframework.stereotype.Component;
import org.springframework.web.filter.OncePerRequestFilter;

/** Adds X-Served-By: <replica name> to every response, so load balancing is visible. */
@Component
public class ServedByFilter extends OncePerRequestFilter {

    private final String replica = Replica.name();

    @Override
    protected void doFilterInternal(HttpServletRequest request, HttpServletResponse response,
            FilterChain chain) throws ServletException, IOException {
        response.setHeader("X-Served-By", replica);
        chain.doFilter(request, response);
    }
}
